use std::collections::HashMap;
use ring::digest::{Context, SHA256};
use data_encoding::{HEXLOWER};
use redis::{RedisResult, AsyncCommands};
use redis::aio::Connection;
use crate::leak_model::LeakModel;
use std::ops::Add;

pub fn load_credentials(path: String) -> HashMap<String, String> {
    let f = std::fs::read(path);
    if let Ok(v) = f {
        let res: Result<HashMap<String, String>, serde_json::Error> =
            serde_json::from_slice(v.as_slice());
        if let Ok(b) = res {
            return b;
        }
        HashMap::new()
    } else {
        HashMap::new()
    }
}

pub fn sha256_encode(b: &[u8]) -> String {
    let mut ctx = Context::new(&SHA256);
    let _ = ctx.update(b);
    let digest = ctx.finish();
    HEXLOWER.encode(digest.as_ref())
}

pub fn md5_encode(b: &[u8]) -> String {
    let hasher = md5::compute(b);
    HEXLOWER.encode(&hasher.0)
}

pub async fn report_leak(conn: &mut Connection, leak: &LeakModel) {
    let _ : RedisResult<()> = conn.set(leak.endpoint_hash.clone().add("_endpoint"), &leak.endpoint).await;
    let secret: String= conn.get(String::from("SECURUM_EXIRE_SIGNAL_SERVER_SECRET")).await.unwrap_or(String::from(""));
    let client = reqwest::Client::new();
    let response = client.request(
        reqwest::Method::POST,
        "http://localhost:9000/report/leak")
        .header("SECRET", secret)
        .json(&leak)
        .send()
        .await;
    match response {
        Ok(r) => {
            colour::e_yellow_ln!("leak reported, signal server responded with: {}",r.status().as_u16())
        }
        Err(e) => {
            colour::e_red_ln!("error: {:?}", e);
        }
    };
}

