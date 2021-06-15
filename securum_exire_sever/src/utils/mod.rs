use std::collections::HashMap;
use ring::digest::{
    Context,
    Digest,
    SHA256
};
use std::io::Read;
use data_encoding::{HEXUPPER, HEXLOWER};

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

