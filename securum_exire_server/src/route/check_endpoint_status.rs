use crate::utils::md5_encode;
use actix_web::{HttpResponse, Responder};
use redis::{AsyncCommands, RedisResult};
use std::ops::Add;

pub async fn check_endpoint_status(
    redis_client: actix_web::web::Data<redis::Client>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    // endpoint has the endpoint hash
    let ep = req.headers().get("endpoint");
    let ep = match ep {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    let endpoint = md5_encode(ep.as_bytes());
    let conn = redis_client.get_async_connection().await;
    let is_blocked = match conn {
        Ok(mut conn) => {
            let result: RedisResult<String> = conn.get(endpoint.add("_blocked_endpoint")).await;
            if let Ok(v) = result {
                colour::e_yellow_ln!("route is blocked: {}", v);
                true
            } else {
                false
            }
        }
        Err(_) => false,
    };
    if is_blocked {
        HttpResponse::Forbidden()
    } else {
        HttpResponse::Ok()
    }
}
