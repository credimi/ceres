use std::io::Result as IoResult;

use actix_web::{get, App, HttpResponse, HttpServer};
use clap::Parser;

use ceres::routes::*;
use ceres::utils::logging::*;

#[actix_web::main]
async fn main() -> IoResult<()> {
    let log = get_root_logger();
    let cli = Cli::parse();

    let listen_addr = std::env::var("HTTP_SERVER_LISTEN_ADDR").expect("Missing required HTTP_SERVER_LISTEN_ADDR");

    info!(log, "Start server @ {}", listen_addr; "listen_addr" => ?listen_addr);
    HttpServer::new(move || {
        App::new()
            .app_data(AppConfig {
                log: log.clone(),
            })
            .service(healthz)
    })
    .bind(listen_addr)?
    .run()
    .await
}

#[get("/api/v1/healthz")]
async fn healthz() -> IoResult<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("application/json").finish())
}
