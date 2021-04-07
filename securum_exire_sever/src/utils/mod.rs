use std::collections::HashMap;
pub fn load_credentials(path: String) -> HashMap<String, String> {
    let f = std::fs::read(path);
    if let Ok(v) = f {
        let res: Result<HashMap<String, String>, serde_json::Error> = serde_json::from_slice(v.as_slice());
        if let Ok(b) = res {
            return b;
        }
        HashMap::new()
    } else {
        HashMap::new()
    }
}