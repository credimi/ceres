use std::io::Result as IoResult;

use crate::auth::CervedOAuthConfig;
use crate::aws::aws_s3::{AwsConf, S3Client};
use crate::qrp::cerved_qrp::CervedQrpClient;
use crate::qrp::{QrpFormat, QrpProduct, QrpRequest, SubjectType};
use crate::utils::logging::Logger;
use actix_web::web::{Data, Path, Query};
use actix_web::{post, HttpResponse};
use base64::Engine;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slog::{error, info};
use uuid::Uuid;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub http_client_config: HttpClientConfig,
    #[command(flatten)]
    pub cerved_oauth_config: CervedOAuthConfig,
    #[command(flatten)]
    pub aws_conf: AwsConf,
}

#[derive(Parser, Debug, Clone)]
pub struct HttpClientConfig {
    #[arg(long, env)]
    pub cerved_base_url: String,
}

pub struct AppConfig {
    pub log: Logger,
    pub cerved_qrp_client: CervedQrpClient,
    pub aws_s3_client: S3Client,
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

    // TODO: should be still used?
    let _ = query.maxTtl;

    let reference = Uuid::now_v7();

    let qrp_req = QrpRequest::builder()
        .reference(reference)
        .product_id(QrpProduct::Qrp)
        .format(QrpFormat::Xml)
        .subject_type(SubjectType::Company)
        .vat_number(Some(vat_number.clone()))
        .tax_code(None)
        .build();

    info!(log, "Requesting QRP XML for user {} with req: {:?}", user, &qrp_req);
    let result = qrp_client.generate_qrp_with_retry(&qrp_req).await;

    match result {
        Ok(res) => {
            let _res = res.clone();
            let data = base64::engine::general_purpose::STANDARD
                .decode(
                    res.content
                        .unwrap_or_else(|| panic!("No content found in response: {:?}", _res)),
                )
                .expect("Invalid base64");

            info!(
                log,
                "Uploading to S3: QRP XML for vat {} by user {} with requestId: {:?}", vat_number, user, res.request_id
            );
            let s3_client = &app_data.aws_s3_client;
            let upload_res = s3_client.upload(&data, &vat_number, &user, QrpFormat::Xml).await;
            match upload_res {
                Ok(_) => {
                    info!(log, "Uploaded QRP XML for vat {}", vat_number)
                }
                Err(_) => {
                    // TODO: should we return an error here?
                    error!(log, "Failed upload QRP XML for vat {}", vat_number)
                }
            }

            info!(
                log,
                "Requesting QRP PDF for user {} with requestId: {:?}", user, res.request_id
            );
            let result_pdf = qrp_client.read_qrp_with_retry(res.request_id, &QrpFormat::Pdf).await;

            match result_pdf {
                Ok(pdf) => {
                    let _pdf = pdf.clone();
                    let data = base64::engine::general_purpose::STANDARD
                        .decode(
                            pdf.content
                                .unwrap_or_else(|| panic!("No content found in response: {:?}", _pdf)),
                        )
                        .expect("Invalid base64");

                    info!(
                        log,
                        "Uploading to S3: QRP PDF for user {} with requestId: {:?}", user, res.request_id
                    );
                    let upload_res = s3_client.upload(&data, &vat_number, &user, QrpFormat::Xml).await;

                    match upload_res {
                        Ok(_) => {
                            info!(log, "Uploaded QRP PDF for vat {}", vat_number)
                        }
                        Err(_) => {
                            // TODO: should we return an error here?
                            error!(log, "Failed upload QRP PDF for vat {}", vat_number)
                        }
                    }

                    Ok(HttpResponse::Created().finish())
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
