mod api;
mod database;
mod model;

use crate::api::tasks::{complete_task, fail_task, start_task, submit_task};
use actix_web::middleware::Logger;
use actix_web::{web::Data, App, HttpServer};
use api::tasks::get_tasks;
use database::db::DB;
use dotenv::dotenv;

const LOCAL_HOST: &str = "127.0.0.1"; // setup local host and port
const LOCAL_PORT: u16 = 8080;

/// Setup the environment logger
///
/// Sets the RUST_LOG and RUST_BACKTRACE environment variables
/// and initializes the logger
///
fn setup_env_logger() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}

/// Main function
///
/// Sets up the environment logger, loads the .env file, and
/// creates the HTTP server
///
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_env_logger();
    dotenv().ok();
    let config = aws_config::load_from_env().await;

    HttpServer::new(move || {
        let ddb_instance = DB::new(String::from("tasks"), config.clone());
        let data = Data::new(ddb_instance);
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(data)
            .service(get_tasks)
            .service(submit_task)
            .service(start_task)
            .service(fail_task)
            .service(complete_task)
    })
    .bind((LOCAL_HOST.to_string(), LOCAL_PORT))?
    .run()
    .await
}
