use crate::route::{check_leak::check,
                   check_endpoint_status::check_endpoint_status,
                   block_endpoint::block_endpoint,
                   get_all_blocked::get_all_blocked,
                   unblock_endpoint::unblock_endpoint,
                   register_signal_server::register_signal_server
};
use crate::utils::{load_credentials, report_leak};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use crate::leak_model::LeakModel;
use futures::future::Either;
use redis::{AsyncCommands, RedisResult};
use crate::watcher::start_watcher;
use crate::config::SecExireConf;

pub async fn start(conf: Arc<Box<SecExireConf>>) -> Result<(), Box<dyn std::error::Error>> {
    let redis_client = redis::Client::open(conf.redis_url.clone())
        .expect("unable to connect to redis server");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let (tx, mut rx) = mpsc::channel::<LeakModel>(1024);
    let cred = load_credentials(conf.secrets_file_path.clone());
    let cred_share_obj = Arc::new(Mutex::new(RefCell::new(cred)));
    let watcher_cred_copy = cred_share_obj.clone();
    let _w = start_watcher(watcher_cred_copy, conf.clone());

    let mut stream = signal(SignalKind::interrupt())?;
    let client_clone = redis_client.clone();
    let conf_clone = conf.clone();
    let handler = tokio::spawn(async move {
        let client = client_clone.clone();
        let conf = conf_clone.clone();
        let conn = client.get_async_connection().await;
        if conn.is_err() {
            colour::e_red_ln!("error: unable to connect to redis server");
            std::process::exit(1);
        }
        let mut conn = conn.unwrap();
        loop {
            let rec = rx.recv();
            let hang = stream.recv();
            futures::pin_mut!(rec);
            futures::pin_mut!(hang);
            match futures::future::select(rec, hang).await {
                Either::Left((e, _)) => {
                    if let Some(v) = e {
                        let _ : RedisResult<()>= conn.set(v.payload_hash.clone(), true).await;
                        report_leak(&mut conn, &v, conf.clone()).await;
                        colour::e_yellow_ln!("leak detected: {:?}", v);
                    }
                }
                Either::Right(_) => {
                    colour::e_green_ln!("hanging up the leaks handling task...");
                    break;
                }
            };
        }
        ()
    });
    let conf_clone = conf.clone();
    let listen_at = &conf.listening_port_address;
    let server = HttpServer::new(move || {
        App::new()
            .data(conf_clone.clone())
            .data(cred_share_obj.clone())
            .data(redis_client.clone())
            .data(tx.clone())
            .wrap(Logger::default())
            .route("/check", web::post().to(check))
            .route("/check_endpoint", web::get().to(check_endpoint_status))
            .route("/block_endpoint", web::get().to(block_endpoint))
            .route("/register_signal_server", web::get().to(register_signal_server))
            .route("/get_all_blocked_endpoints", web::get().to(get_all_blocked))
            .route("/unblock_endpoint", web::get().to(unblock_endpoint))
    })
    .bind(listen_at)?
    .run();
    let _ = futures::future::join(server, handler).await;
    Ok(())
}
