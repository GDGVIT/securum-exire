use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct Conf {
    production: Option<SecExireConf>,
    development: Option<SecExireConf>,
    staging: Option<SecExireConf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SecExireConf {
    pub redis_url: String,
    pub secrets_file_path: String,
    pub listening_port_address: String,
    pub signal_server_address: String,
}

fn sanitize_conf(conf: &Box<SecExireConf>) -> (bool, String) {
    let secrets_file_path: String = conf.secrets_file_path.clone();
    let listening_port: String = conf.listening_port_address.clone();
    let signal_server_address: String = conf.signal_server_address.clone();
    let listening_address_regex =
        regex::Regex::new("^\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}:\\d{1,5}$").unwrap();
    let signal_server_address_regex =
        regex::Regex::new("^[\\.a-zA-Z0-9\\-]+:{0,1}\\d{0,5}$").unwrap();

    if !listening_address_regex.is_match(listening_port.as_str()) {
        return (
            false,
            "error: invalid listening_port_address [USAGE: 0.0.0.0:9000]".to_string(),
        );
    }
    if !signal_server_address_regex.is_match(signal_server_address.as_str()) {
        return (
            false,
            "error: invalid signal_server_address [USAGE: http://localhost:9000]".to_string(),
        );
    }
    if !std::path::Path::new(&secrets_file_path).exists() {
        return (
            false,
            format!(
                "error: path[{}] secrets_file_path doesn't exist",
                secrets_file_path
            )
            .to_string(),
        );
    }

    (true, "successful".to_string())
}

fn check_success(conf: Box<SecExireConf>) -> Arc<Box<SecExireConf>> {
    let (success, msg) = sanitize_conf(&conf);
    if !success {
        colour::e_red_ln!("{}", msg);
        std::process::exit(1);
    }
    return Arc::new(conf);
}

pub fn load_conf(env: String, config_loc: String) -> Arc<Box<SecExireConf>> {
    let res = std::fs::read_to_string(config_loc);
    if res.is_err() {
        colour::e_red_ln!("error: unable to open config file");
        std::process::exit(1);
    }
    let content = res.unwrap();
    let cnf: Result<Conf, toml::de::Error> = toml::from_str(content.as_str());
    if cnf.is_err() {
        colour::e_red_ln!("error: malformed config structure");
        std::process::exit(1);
    }
    let cnf = cnf.unwrap();
    if env.to_ascii_uppercase() == "PRODUCTION" {
        if cnf.production.is_none() {
            colour::e_red_ln!("error: production credentials not found!");
            std::process::exit(1);
        }
        let conf = Box::new(cnf.production.unwrap());
        return check_success(conf);
    } else if env.to_ascii_uppercase() == "DEVELOPMENT" {
        if cnf.development.is_none() {
            colour::e_red_ln!("error: development credentials not found!");
            std::process::exit(1);
        }
        let conf = Box::new(cnf.development.unwrap());
        return check_success(conf);
    } else if env.to_ascii_uppercase() == "STAGING" {
        if cnf.development.is_none() {
            colour::e_red_ln!("error: development credentials not found!");
            std::process::exit(1);
        }
        let conf = Box::new(cnf.staging.unwrap());
        return check_success(conf);
    }
    colour::e_red_ln!("error: invalid env type flag [development/staging/production]");
    std::process::exit(1);
}
