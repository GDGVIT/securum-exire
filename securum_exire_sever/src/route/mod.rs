use crate::leak_model::LeakModel;
use actix_web::{web, HttpResponse, Responder};
use futures_util::stream::StreamExt as _;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use ring::digest::{
    Context,
    Digest,
    SHA256
};
use crate::utils::sha256_encode;
use redis::Commands;


pub async fn check(
    data: actix_web::web::Data<Arc<Mutex<RefCell<HashMap<String, String>>>>>,
    _chan: actix_web::web::Data<tokio::sync::mpsc::Sender<LeakModel>>,
    redis_client : actix_web::web::Data<redis::Client>,
    mut payload: web::Payload,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let data = data.deref().lock().unwrap();
    let data = data.borrow();
    let keys = data.keys();

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        if let Ok(chunk) = chunk {
            body.extend_from_slice(&chunk);
        } else {
            break;
        }
    }
    let c = &body[..];
    let sha256_hash = sha256_encode(c);
    let conn = redis_client.get_connection();
    let is_leak = match conn {
        Ok(mut conn) => {
            conn.get(sha256_hash.clone()).unwrap_or(false)
        },
        Err(_) => {
            false
        }
    };

    if is_leak {
        return HttpResponse::Forbidden();
    }

    let req_payload = String::from_utf8(Vec::from(c)).unwrap();
    let mut f = false;

    let z = keys
        .map(|i| {
            let c = data.get(i).unwrap().clone();
            let d = req_payload.clone();
            let key = i.clone();
            tokio::task::spawn(async move {
                let x = c;
                let b = d.contains(&x);
                return (b, key);
            })
        })
        .collect::<Vec<JoinHandle<(bool, String)>>>();
    let result = futures::future::join_all(z).await;

    let mut leaks = Vec::new();
    for i in result {
        if let Ok(v) = i {
            if v.0 {
                leaks.push(v.1);
            }
            f = f || v.0;
        }
    }

    let endpoint = req.headers().get("endpoint");
    let endpoint = match endpoint {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    if f {
        let _ = _chan
            .send(LeakModel {
                endpoint: endpoint.to_string(),
                leaked_credentials: leaks,
                payload_hash: sha256_hash
            })
            .await;
        HttpResponse::Forbidden()
    } else {
        HttpResponse::Ok()
    }
}
