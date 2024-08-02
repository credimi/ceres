use std::io::Result as IoResult;

use actix_web::{get, App, HttpResponse, HttpServer};
use clap::Parser;

use rust_microservice::routes::*;
use rust_microservice::utils::logging::*;

#[actix_web::main]
async fn main() -> IoResult<()> {
    let log = get_root_logger();
    let cli = Cli::parse();

    let listen_addr = std::env::var("HTTP_SERVER_LISTEN_ADDR").expect("Missing required HTTP_SERVER_LISTEN_ADDR");

    info!(log, "Start server @ {}", listen_addr; "listen_addr" => ?listen_addr);
    let db = cli.db_config.connect().unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(AppConfig {
                db: db.clone(),
                log: log.clone(),
            })
            .service(healthz)
            .service(insert_flag)
            .service(get_flag)
    })
    .bind(listen_addr)?
    .run()
    .await
}

#[get("/api/v1/healthz")]
async fn healthz() -> IoResult<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("application/json").finish())
}
