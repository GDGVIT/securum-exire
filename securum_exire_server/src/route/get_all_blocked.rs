use actix_web::{HttpResponse, Responder};
use std::collections::BTreeMap;
use std::ops::Add;

pub async fn get_all_blocked(redis_client: actix_web::web::Data<redis::Client>) -> impl Responder {
    let client = redis_client.get_async_connection().await;
    if let Ok(mut con) = client {
        let endpoint_keys: Vec<String> = redis::cmd("KEYS")
            .arg("*_blocked_endpoint")
            .query_async(&mut con)
            .await
            .unwrap_or(vec![]);
        let endpoint_keys = endpoint_keys
            .into_iter()
            .map(|v| v.chars().take(32).collect::<String>().add("_endpoint"))
            .collect::<Vec<String>>();
        let endpoints: Vec<String> = redis::cmd("MGET")
            .arg(endpoint_keys.clone())
            .query_async(&mut con)
            .await
            .unwrap_or(vec![]);

        let mut payload: BTreeMap<String, String> = BTreeMap::new();
        for i in 0..(endpoint_keys.len()) {
            payload.insert(
                endpoint_keys[i].chars().take(32).collect::<String>(),
                endpoints[i].clone(),
            );
        }

        HttpResponse::Ok().json(payload)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
