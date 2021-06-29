use crate::utils::load_credentials;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use hotwatch::Event;

pub fn start_watcher(watcher_cred_copy: Arc<Mutex<RefCell<HashMap<String, String>>>>) {
    let mut watcher = hotwatch::Hotwatch::new().expect("watcher failed to initialize");
    let x = watcher.watch("./credentials.json", move |_e: Event| {
        let cr = watcher_cred_copy.clone();
        let b = cr.lock().unwrap();
        colour::e_yellow_ln!("credentials changed updating the path!");
        let c = load_credentials("./credentials.json".to_string());
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