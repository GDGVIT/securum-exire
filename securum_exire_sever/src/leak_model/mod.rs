#[derive(Debug)]
pub struct LeakModel {
    pub endpoint: String,
    pub leaked_credentials: Vec<String>,
    pub payload_hash: String
}
impl LeakModel {
    fn create_bot_message(self) -> String {
        format!(
            "Leaks for the following params{:?} have occurred at:\n {}",
            self.leaked_credentials, self.endpoint
        )
    }
}
