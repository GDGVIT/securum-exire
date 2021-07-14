use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct LeakModel {
    pub endpoint: String,
    pub leaked_credentials: Vec<String>,
    pub payload_hash: String,
    pub endpoint_hash: String,
}
