use crate::route::{
    block_endpoint::block_endpoint, check_endpoint_status::check_endpoint_status,
    check_leak::check, get_all_blocked::get_all_blocked,
    register_signal_server::register_signal_server, unblock_endpoint::unblock_endpoint,
};
use crate::utils::{heartbeat, load_credentials, report_leak};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};

use crate::config::SecExireConf;
use crate::leak_model::LeakModel;
use crate::watcher::start_watcher;
use futures::future::FutureExt;
use redis::{AsyncCommands, RedisResult};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
pub async fn start(conf: Arc<Box<SecExireConf>>) -> Result<(), Box<dyn std::error::Error>> {
    let redis_client =
        redis::Client::open(conf.redis_url.clone()).expect("unable to connect to redis server");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let (tx, mut rx) = mpsc::channel::<LeakModel>(1024);
    let cred = load_credentials(conf.secrets_file_path.clone());
    let cred_share_obj = Arc::new(Mutex::new(RefCell::new(cred)));
    let watcher_cred_copy = cred_share_obj.clone();
    let _w = start_watcher(watcher_cred_copy, conf.clone());

    let mut term_signal = signal(SignalKind::terminate())?;
    let mut hang_signal = signal(SignalKind::hangup())?;
    let mut quit_signal = signal(SignalKind::quit())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;

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
            let rec = rx.recv().fuse();
            let term_sig = term_signal.recv().fuse();
            let int_sig = interrupt_signal.recv().fuse();
            let quit_sig = quit_signal.recv().fuse();
            let hang_sig = hang_signal.recv().fuse();
            futures::pin_mut!(term_sig, int_sig, quit_sig, hang_sig, rec);
            futures::select! {
                e = rec => {
                    if let Some(v) = e {
                        let _ : RedisResult<()>= conn.set(v.payload_hash.clone(), true).await;
                        report_leak(&mut conn, &v, conf.clone()).await;
                        colour::e_yellow_ln!("leak detected: {:?}", v);
                    }
                },
                _ = hang_sig => {
                    colour::e_green_ln!("quiting leaks handling task...");
                    break;
                },
                _ = term_sig => {
                    colour::e_green_ln!("quiting leaks handling task...");
                    break;
                },
                _ = quit_sig => {
                    colour::e_green_ln!("quiting leaks handling task...");
                    break;
                },
                _ = int_sig => {
                    colour::e_green_ln!("quiting leaks handling task...");
                    break;
                }
            };
        }
        ()
    });
    let conf_clone = conf.clone();
    heartbeat(conf.clone()).await;
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
            .route(
                "/register_signal_server",
                web::get().to(register_signal_server),
            )
            .route("/get_all_blocked_endpoints", web::get().to(get_all_blocked))
            .route("/unblock_endpoint", web::get().to(unblock_endpoint))
    })
    .bind(listen_at)?
    .run();
    let _ = futures::future::join(server, handler).await;
    Ok(())
}
