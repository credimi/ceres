use std::io::Result as IoResult;

use actix_web::web::Data;
use actix_web::{get, App, HttpResponse, HttpServer};
use ceres::qrp::cerved_qrp::CervedQrpClient;
use ceres::routes::*;
use ceres::utils::logging::*;
use clap::Parser;

#[actix_web::main]
async fn main() -> IoResult<()> {
    let log = get_root_logger();
    let cli = Cli::parse();
    let http_client_config = cli.http_client_config;
    let cerved_oauth_config = cli.cerved_oauth_config;

    let listen_addr = std::env::var("HTTP_SERVER_LISTEN_ADDR").expect("Missing required HTTP_SERVER_LISTEN_ADDR");

    info!(log, "Start server @ {}", listen_addr; "listen_addr" => ?listen_addr);

    let http_client = reqwest::Client::new();
    let cerved_qrp_client =
        CervedQrpClient::new(http_client, &http_client_config.cerved_base_url, &cerved_oauth_config).await;
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppConfig {
                log: log.clone(),
                cerved_qrp_client: cerved_qrp_client.clone(),
            }))
            .service(healthz)
            .service(call_cerved_qrp)
    })
    .bind(listen_addr)?
    .run()
    .await
}

#[get("/api/v1/healthz")]
async fn healthz() -> IoResult<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("application/json").finish())
}
