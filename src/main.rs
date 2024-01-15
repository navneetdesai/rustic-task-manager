mod api;
mod database;
mod model;

use api::tasks::get_tasks;
use actix_web::{HttpServer, App, web::Data, Responder, HttpResponse};
use actix_web::middleware::Logger;
use database::db::DB;

fn setup_env_logger() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_env_logger();
    let config = aws_config::load_from_env().await;

    HttpServer::new(move || {
        let ddb_instance = DB::new(String::from("tasks"), config.clone());
        let data = Data::new(ddb_instance);
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(data)
            .service(get_tasks)
    }).bind(("127.0.0.1", 8080))?
        .run()
        .await
}
