mod init;
mod leak_model;
mod route;
mod utils;
use init::start;

#[actix_web::main]
async fn main() {
    match start().await {
        Ok(_) => {}
        Err(_) => {
            println!("error occurred while starting the server!")
        }
    }
}
