use actix_web::{HttpResponse, Responder};
use redis::{AsyncCommands, RedisResult};

pub async fn register_signal_server(
    redis_client: actix_web::web::Data<redis::Client>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let secret = req.headers().get("secret");
    let secret = match secret {
        Some(v) => v.to_str().unwrap(),
        None => "",
    };
    let conn = redis_client.get_async_connection().await;
    return match conn {
        Ok(mut conn) => {
            let result: RedisResult<()> =
                conn.set("SECURUM_EXIRE_SIGNAL_SERVER_SECRET", secret).await;
            if let Ok(_) = result {
                HttpResponse::Ok()
            } else {
                HttpResponse::InternalServerError()
            }
        }
        Err(_) => HttpResponse::InternalServerError(),
    };
}
