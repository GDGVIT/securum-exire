use actix_web::{HttpResponse, Responder};
use redis::{AsyncCommands, RedisResult};
use std::ops::Add;

pub async fn block_endpoint(
    redis_client: actix_web::web::Data<redis::Client>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let endpoint = req.headers().get("endpoint");

    let endpoint = match endpoint {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    let conn = redis_client.get_async_connection().await;
    if let Ok(mut conn) = conn {
        let _: RedisResult<()> = conn
            .set(
                (String::from(endpoint)).add("_blocked_endpoint"),
                String::from(endpoint),
            )
            .await;
        HttpResponse::Ok()
    } else {
        HttpResponse::InternalServerError()
    }
}
