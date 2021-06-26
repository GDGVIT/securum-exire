use crate::route::{check, check_endpoint_status};
use crate::utils::load_credentials;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use hotwatch::Event;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;

use crate::leak_model::LeakModel;
use futures::future::Either;
use std::collections::HashMap;
use redis::{Commands, RedisResult};
use std::ops::Add;
use reqwest::{Response, Error};

fn start_watcher(watcher_cred_copy: Arc<Mutex<RefCell<HashMap<String, String>>>>) {
    let mut watcher = hotwatch::Hotwatch::new().expect("watcher failed to initialize");
    let x = watcher.watch("./credentials.json", move |_e: Event| {
        let cr = watcher_cred_copy.clone();
        let b = cr.lock().unwrap();
        println!("credentials changed updating the path!");
        let c = load_credentials("./credentials.json".to_string());
        let mut data = b.borrow_mut();
        for (k, v) in c {
            data.insert(k, v);
        }
    });
    if let Ok(_) = x {
        eprintln!("watcher started...");
    } else {
        eprintln!("watcher thread failed to initialize...");
    }
}
// TODO: Move to utils
async fn report_leak(leak: &LeakModel) {
    // let payload = serde_json::to_string(&leak).unwrap_or("{}".into());

    let client = reqwest::Client::new();
    // let request =
    let response = client.request(reqwest::Method::POST,
                                  "http://localhost:9000/report/leak")
        .json(&leak)
        .send()
        .await;
    match response {
        Ok(r) => {
           println!("leak reported, signal server responded with: {}",r.status().as_u16())
        }
        Err(e) => {
            println!("error: {:?}", e);
        }
    };
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let redis_client = redis::Client::open("redis://localhost")
        .expect("unable to connect to redis server");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let (tx, mut rx) = mpsc::channel::<LeakModel>(1024);
    let cred = load_credentials("./credentials.json".to_string());
    let cred_share_obj = Arc::new(Mutex::new(RefCell::new(cred)));
    let watcher_cred_copy = cred_share_obj.clone();
    start_watcher(watcher_cred_copy);

    let mut stream = signal(SignalKind::interrupt())?;
    let client_clone = redis_client.clone();
    let handler = tokio::spawn(async move {
        let client = client_clone.clone();
        let mut conn = client.get_connection().expect("unable to connect to redis!");
        loop {
            let rec = rx.recv();
            let hang = stream.recv();
            futures::pin_mut!(rec);
            futures::pin_mut!(hang);
            match futures::future::select(rec, hang).await {
                Either::Left((e, _)) => {
                    if let Some(v) = e {
                        let _ : RedisResult<()>= conn.set(v.payload_hash.clone(), true);
                        let _ : RedisResult<()>= conn.set(v.endpoint.clone().add("_blocked_endpoint"), true);
                        report_leak(&v).await;
                        println!("leak detected: {:?}", v);
                    }
                }
                Either::Right(_) => {
                    println!("hanging up the leaks handling task...");
                    break;
                }
            };
        }
        ()
    });

    let server = HttpServer::new(move || {
        App::new()
            .data(cred_share_obj.clone())
            .data(redis_client.clone())
            .data(tx.clone())
            .wrap(Logger::default())
            .route("/check", web::post().to(check))
            .route("/check_endpoint", web::get().to(check_endpoint_status))
    })
    .bind("0.0.0.0:8080")?
    .run();
    let _ = futures::future::join(server, handler).await;
    Ok(())
}
