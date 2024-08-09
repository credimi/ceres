use std::error::Error;
use std::fmt::Debug;
use std::io::Result as IoResult;

use crate::auth::CervedOAuthConfig;
use crate::aws::aws_s3::{AwsConf, S3Client};
use crate::qrp::cerved_qrp::CervedQrpClient;
use crate::qrp::qrp_service::request_qrp;
use crate::qrp::{QrpFormat, QrpProduct, QrpRequest, SubjectType};
use actix_web::web::{Data, Path, Query};
use actix_web::{get, post, HttpResponse};
use clap::Parser;
use serde::Deserialize;
use tracing::info;
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
    pub cerved_api_base_url: String,
}

pub struct AppConfig {
    pub cerved_qrp_client: CervedQrpClient,
    pub aws_s3_client: S3Client,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct QrpQuery {
    user: String,
}

#[post("/qrp/{vat_number}")]
pub async fn generate_cerved_qrp(
    app_data: Data<AppConfig>,
    path: Path<String>,
    query: Query<QrpQuery>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let qrp_client = app_data.cerved_qrp_client.clone();
    let s3_client = app_data.aws_s3_client.clone();

    let vat_number = path.into_inner();
    let user = query.user.clone();

    let reference = Uuid::now_v7();

    let qrp_req = QrpRequest {
        reference,
        product_id: QrpProduct::Qrp,
        format: QrpFormat::Xml,
        subject_type: SubjectType::Company,
        vat_number: Some(vat_number.clone()),
        tax_code: None,
    };

    info!("Requesting QRP XML for user {} with req: {:?}", user, &qrp_req);
    request_qrp(qrp_client, s3_client, vat_number, user, &qrp_req).await?
}

#[get("/api/v1/healthz")]
async fn healthz() -> IoResult<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("application/json").finish())
}
