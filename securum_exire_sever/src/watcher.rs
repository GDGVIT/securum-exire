use crate::utils::load_credentials;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use hotwatch::Event;
use crate::config::SecExireConf;

pub fn start_watcher(watcher_cred_copy: Arc<Mutex<RefCell<HashMap<String, String>>>>, conf: Arc<Box<SecExireConf>>) {
    let mut watcher = hotwatch::Hotwatch::new().expect("watcher failed to initialize");
    let path = conf.secrets_file_path.clone();
    let x = watcher.watch(path.clone(), move |_e: Event| {
        let cr = watcher_cred_copy.clone();
        let b = cr.lock().unwrap();
        colour::e_yellow_ln!("credentials changed updating the path!");
        let c = load_credentials(path.clone());
        let mut data = b.borrow_mut();
        for (k, v) in c {
            data.insert(k, v);
        }
    });
    if let Ok(_) = x {
        colour::e_yellow_ln!("watcher started...");
    } else {
        colour::e_red_ln!("watcher thread failed to initialize...");
    }
}