use std::io::Result as IoResult;

use actix_web::web::Data;
use actix_web::{get, App, HttpResponse, HttpServer};
use ceres::aws::aws_s3::{AwsConf, S3Client};
use ceres::qrp::cerved_qrp::CervedQrpClient;
use ceres::routes::*;
use clap::Parser;
use tracing::info;

#[actix_web::main]
async fn main() -> IoResult<()> {
    tracing_subscriber::fmt().json().init();
    
    let cli = Cli::parse();
    let http_client_config = cli.http_client_config;
    let cerved_oauth_config = cli.cerved_oauth_config;

    let listen_addr = std::env::var("HTTP_SERVER_LISTEN_ADDR").expect("Missing required HTTP_SERVER_LISTEN_ADDR");

    info!("Ceres C'è @ {}", listen_addr);

    let http_client = reqwest::Client::new();
    let cerved_qrp_client = CervedQrpClient::new(
        http_client,
        &http_client_config.cerved_api_base_url,
        &cerved_oauth_config,
    ) 
    .await;

    let aws_conf = AwsConf {
        qrp_bucket_name: cli.aws_conf.qrp_bucket_name,
        s3_dry_run: cli.aws_conf.s3_dry_run,
    };
    let s3_client = S3Client::from_env(aws_conf)
        .await
        .expect("Failed to create AWS S3 configuration");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppConfig {
                cerved_qrp_client: cerved_qrp_client.clone(),
                aws_s3_client: s3_client.clone(),
            }))
            .service(healthz)
            .service(generate_cerved_qrp)
    })
    .bind(listen_addr)?
    .run()
    .await
}

#[get("/api/v1/healthz")]
async fn healthz() -> IoResult<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("application/json").finish())
}
