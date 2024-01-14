mod api;

use api::tasks::get_tasks;
use actix_web::{HttpServer, App, web, Responder, HttpResponse};
use actix_web::middleware::Logger;
fn setup_env_logger() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_env_logger();
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(get_tasks)
    }).bind(("127.0.0.1", 80))?
        .run()
        .await
}
