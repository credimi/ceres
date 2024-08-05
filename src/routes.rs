use std::io::Result as IoResult;

use crate::auth::CervedOAuthConfig;
use crate::qrp::cerved_qrp::CervedQrpClient;
use crate::qrp::{QrpFormat, QrpProduct, QrpRequest, SubjectType};
use crate::utils::logging::Logger;
use actix_web::web::{Data, Path, Query};
use actix_web::{post, HttpResponse};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slog::info;
use uuid::Uuid;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub http_client_config: HttpClientConfig,
    #[command(flatten)]
    pub cerved_oauth_config: CervedOAuthConfig,
}

#[derive(Parser, Debug, Clone)]
pub struct HttpClientConfig {
    #[arg(long, env)]
    pub cerved_base_url: String,
}

pub struct AppConfig {
    pub log: Logger,
    pub cerved_qrp_client: CervedQrpClient,
}

#[derive(Serialize)]
pub enum ErrorKind {
    DatabaseError,
}

#[derive(Serialize)]
pub struct InternalError {
    pub kind: ErrorKind,
    pub reason: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct QrpQuery {
    user: String,
    maxTtl: u32,
}

#[post("/qrp/{vat_number}")]
pub async fn call_cerved_qrp(
    app_data: Data<AppConfig>,
    path: Path<String>,
    query: Query<QrpQuery>,
) -> IoResult<HttpResponse> {
    let log = &app_data.log;
    let qrp_client = &app_data.cerved_qrp_client;

    let vat_number = path.into_inner();
    let user = query.user.clone();
    let _ = query.maxTtl;

    let reference = Uuid::now_v7();

    let qrp_req = QrpRequest::builder()
        .reference(reference)
        .product_id(QrpProduct::QRP)
        .format(QrpFormat::XML)
        .subject_type(SubjectType::COMPANY)
        .vat_number(Some(vat_number.clone()))
        .tax_code(None)
        .build();

    info!(log, "Requesting QRP XML for user {} with req: {:?}", user, &qrp_req);
    let result = qrp_client.generate_qrp_with_retry(&qrp_req).await;

    match result {
        Ok(res) => {
            // TODO: save XML on S3
            info!(
                log,
                "Requesting QRP PDF for user {} with requestId: {:?}", user, res.request_id
            );
            let result_pdf = qrp_client.read_qrp_with_retry(res.request_id, &QrpFormat::PDF).await;
            match result_pdf {
                Ok(pdf) => {
                    // TODO: save PDF on S3
                    Ok(HttpResponse::Ok().json(pdf))
                }
                Err(_) => Ok(HttpResponse::BadGateway()
                    .json(json!({ "message": "unable to retrieve PDF", "reference":  reference }))),
            }
        }
        Err(_) => Ok(
            HttpResponse::BadGateway().json(json!({ "message": "unable to retrieve XML", "reference":  reference }))
        ),
    }
}
