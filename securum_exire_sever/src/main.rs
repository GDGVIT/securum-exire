mod init;
mod leak_model;
mod route;
mod utils;
mod watcher;
use init::start;

#[actix_web::main]
async fn main() {
    match start().await {
        Ok(_) => {}
        Err(_) => {
            colour::e_red_ln!("error occurred while starting the server!")
        }
    }
}
