use actix_web::{HttpResponse, Responder};
use redis::RedisResult;

pub async fn unblock_endpoint(
    redis_client: actix_web::web::Data<redis::Client>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let endpoint_hash = req.headers().get("endpoint_hash");
    let endpoint_hash = match endpoint_hash {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    let con = redis_client.get_async_connection().await;

    if let Ok(mut con) = con {
        let endpoint_keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("*{}*_blocked_endpoint", endpoint_hash))
            .query_async(&mut con)
            .await
            .unwrap_or(vec![]);
        let result: RedisResult<u64> = redis::cmd("DEL")
            .arg(endpoint_keys)
            .query_async(&mut con)
            .await;
        if let Ok(_) = result {
            HttpResponse::Ok().finish()
        } else {
            HttpResponse::NotFound().finish()
        }
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
