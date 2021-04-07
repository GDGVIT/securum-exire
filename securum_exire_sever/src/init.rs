use crate::utils::load_credentials;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use hotwatch::Event;
use crate::route::check;
use actix_web::{web, HttpServer, App};
use actix_web::middleware::Logger;
use tokio::sync::mpsc;
use tokio::signal::unix::{signal, SignalKind};

use crate::leak_model::LeakModel;
use futures::future::Either;
use std::collections::HashMap;

fn start_watcher(watcher_cred_copy: Arc<Mutex<RefCell<HashMap<String, String>>>>) {
    let mut watcher = hotwatch::Hotwatch::new().expect("watcher failed to initialize");
    let x =watcher.watch("./credentials.json", move |_e: Event| {
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


pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let (tx, mut rx) = mpsc::channel::<LeakModel>(1024);
    let cred = load_credentials("./credentials.json".to_string());
    let cred_share_obj = Arc::new(Mutex::new(RefCell::new(cred)));
    let watcher_cred_copy = cred_share_obj.clone();
    start_watcher(watcher_cred_copy);

    let mut stream = signal(SignalKind::interrupt())?;

    let handler = tokio::spawn(async move {
        loop {
            let rec = rx.recv();
            let hang = stream.recv();
            futures::pin_mut!(rec);
            futures::pin_mut!(hang);
            match futures::future::select(rec, hang).await {
                Either::Left((e, _)) => {
                    if let Some(v) = e {
                        println!("{:?}", v);
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
            .data(tx.clone())
            .wrap(Logger::default())
            .route("/check", web::post().to(check))
    })
        .bind("0.0.0.0:8080")?
        .run();
    let _ = futures::future::join(server, handler).await;
    Ok(())
}
