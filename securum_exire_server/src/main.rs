use crate::config::load_conf;
use crate::init::start;

mod init;
mod leak_model;
mod route;
mod utils;
mod watcher;
pub mod config;
mod cmd;

#[actix_web::main]
async fn main() {
    let config_path = cmd::check_env();
    let deployment_env = cmd::parse_flags();
    let conf = load_conf(deployment_env, config_path);
    match start(conf).await {
        Ok(_) => {}
        Err(_) => {
            colour::e_red_ln!("error occurred while starting the server!")
        }
    }
}
