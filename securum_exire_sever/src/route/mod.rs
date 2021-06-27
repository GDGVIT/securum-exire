use crate::leak_model::LeakModel;
use actix_web::{web, HttpResponse, Responder};
use futures_util::stream::StreamExt as _;
use std::cell::RefCell;
use std::collections::{HashMap, BTreeMap};
use std::ops::{Deref, Add};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use crate::utils::{sha256_encode, md5_encode};
use redis::{AsyncCommands, RedisResult};
use std::result::Result::Ok;
use serde::de::Unexpected::Map;

pub async fn register_signal_server(redis_client : actix_web::web::Data<redis::Client>, req: actix_web::HttpRequest)
-> impl Responder {
    let secret = req.headers().get("secret");
    let secret = match secret {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    println!("{}", secret);
    let conn = redis_client.get_async_connection().await;
    return match conn {
        Ok(mut conn) => {
            let result: RedisResult<()> = conn.set("SECURUM_EXIRE_SIGNAL_SERVER_SECRET", secret).await;
            if let Ok(_) = result { HttpResponse::Ok() }
            else { HttpResponse::InternalServerError() }
        },
        Err(_) => {
            HttpResponse::InternalServerError()
        }
    };
}
pub async fn get_all_blocked(redis_client : actix_web::web::Data<redis::Client>) -> impl Responder {
    let client = redis_client.get_async_connection().await;
    if let Ok(mut con) = client {
        let endpoint_keys: Vec<String>=redis::cmd("KEYS").arg("*_blocked_endpoint").query_async(&mut con).await.unwrap_or(vec![]);
        let endpoint_keys = endpoint_keys.into_iter().map(| v| v.chars().take(32)
            .collect::<String>().add("_endpoint")).collect::<Vec<String>>();
        let endpoints: Vec<String>=redis::cmd("MGET").arg(
            endpoint_keys.clone()).query_async(&mut con).await.unwrap_or(vec![]);

        let mut payload: BTreeMap<String, String> = BTreeMap::new();
        for i in 0..(endpoint_keys.len()) {
            payload.insert(endpoint_keys[i].chars().take(32).collect::<String>(), endpoints[i].clone());
        }

        HttpResponse::Ok()
            .json(payload)
    } else {
        HttpResponse::InternalServerError()
            .finish()
    }

    // _blocked_endpoint
}

pub async fn check_endpoint_status(redis_client : actix_web::web::Data<redis::Client>, req: actix_web::HttpRequest)
                                   -> impl Responder {
    // endpoint has the endpoint hash
    let ep = req.headers().get("endpoint");
    let ep = match ep {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    println!("{}", ep);

    let endpoint = md5_encode(ep.as_bytes());
    let conn = redis_client.get_async_connection().await;
    let is_blocked = match conn {
        Ok(mut conn) => {
            let result : RedisResult<String> = conn.get(endpoint.add("_blocked_endpoint")).await;
            if let Ok(v) = result {
                println!("route is blocked: {}", v);
                true
            } else {
                false
            }
        },
        Err(_) => {
            false
        }
    };
    if is_blocked {
        HttpResponse::Forbidden()
    } else {
        HttpResponse::Ok()
    }

}
pub async fn block_endpoint(redis_client : actix_web::web::Data<redis::Client>, req: actix_web::HttpRequest)
                                   -> impl Responder {
    let endpoint = req.headers().get("endpoint");

    let endpoint = match endpoint {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    let conn = redis_client.get_async_connection().await;
    if let Ok(mut conn) = conn {
        let _ : RedisResult<()>= conn.set(
            (String::from(endpoint)).add("_blocked_endpoint"), String::from(endpoint)).await;
        HttpResponse::Ok()
    } else {
        HttpResponse::InternalServerError()
    }

}

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
