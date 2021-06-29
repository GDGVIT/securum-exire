use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::leak_model::LeakModel;
use std::cell::RefCell;
use actix_web::{web, Responder, HttpResponse};
use std::ops::Deref;
use redis::AsyncCommands;
use crate::utils::{md5_encode, sha256_encode};
use futures_util::StreamExt;
use tokio::task::JoinHandle;

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
    let endpoint = req.headers().get("endpoint");
    let endpoint = match endpoint {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };

    let endpoint_hash = md5_encode(endpoint.as_bytes());

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
    let conn = redis_client.get_async_connection().await;
    let is_leak = match conn {
        Ok(mut conn) => {
            conn.get(sha256_hash.clone()).await.unwrap_or(false)
        },
        Err(_) => {
            false
        }
    };

    if is_leak {
        let _ = _chan
            .send(LeakModel {
                endpoint: endpoint.to_string(),
                leaked_credentials: vec![],
                payload_hash: sha256_hash,
                endpoint_hash
            })
            .await;
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

    if f {
        let _ = _chan
            .send(LeakModel {
                endpoint: endpoint.to_string(),
                leaked_credentials: leaks,
                payload_hash: sha256_hash,
                endpoint_hash
            })
            .await;
        HttpResponse::Forbidden()
    } else {
        HttpResponse::Ok()
    }
}
