use std::fmt::Debug;
use std::io::Result as IoResult;

use crate::auth::CervedOAuthConfig;
use crate::aws::aws_s3::{AwsConf, S3Client};
use crate::errors::ErrorKind::{CervedError, S3Error};
use crate::qrp::cerved_qrp::CervedQrpClient;
use crate::qrp::{QrpFormat, QrpProduct, QrpRequest, QrpResponse, SubjectType};
use actix_web::web::{Data, Path, Query};
use actix_web::{get, post, HttpResponse};
use chrono::Utc;
use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info};
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
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
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
    let result = qrp_client.generate_qrp(&qrp_req).await;

    match result {
        Ok(res) => Ok(get_and_upload(qrp_client, s3_client, &vat_number, &user, &res).await?),
        Err(_) => {
            // TODO: save the request on db and try later
            tokio::spawn(async move {
                let res = qrp_client
                    .generate_qrp_with_retry(&qrp_req)
                    .await
                    .expect("Failed to generate QRP");
                get_and_upload(qrp_client, s3_client, &vat_number, &user, &res)
                    .await
                    .expect("Failed to upload QRP");
            });

            Ok(HttpResponse::Accepted().json(json!({"message": "QRP generation requested"})))
        }
    }
}

async fn get_and_upload(
    qrp_client: CervedQrpClient,
    s3_client: S3Client,
    vat_number: &String,
    user: &String,
    res: &QrpResponse,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let xml_url = content_upload(&s3_client, vat_number, user, res).await.map_err(|e| {
        error!("Failed to upload QRP XML for vat {}: {:?}", vat_number, e);
        S3Error
    })?;

    info!(
        "Requesting QRP PDF for user {} with requestId: {:?}",
        user, res.request_id
    );
    let pdf = qrp_client
        .read_qrp_with_retry(res.request_id, QrpFormat::Pdf)
        .await
        .map_err(|e| {
            error!("Failed to generate QRP PDF for vat {}: {:?}", vat_number, e);
            CervedError
        })?;
    let pdf_url = content_upload(&s3_client, vat_number, user, &pdf).await.map_err(|e| {
        error!("Failed to upload QRP PDF for vat {}: {:?}", vat_number, e);
        S3Error
    })?;

    Ok(HttpResponse::Created().json(json!({"pdf_url": pdf_url, "xml_url": xml_url})))
}

async fn content_upload(
    s3_client: &S3Client,
    vat_number: &String,
    user: &String,
    res: &QrpResponse,
) -> anyhow::Result<String> {
    info!(
        "Uploading to S3: QRP {} for vat {} by user {} with requestId: {:?}",
        res.format, vat_number, user, res.request_id
    );
    let data = res.decode_content();
    let now = Utc::now();
    let date_time = now.format("%d_%m_%Y_%H:%M:%S");
    let lower_case_format = res.format.as_str();
    let file_name = format!("qrp/{vat_number}/{date_time}_{user}.{lower_case_format}");

    let upload_res = s3_client.upload(&data, &file_name).await;
    match upload_res {
        Ok(_) => {
            info!("Uploaded QRP {} for vat {}", res.format, vat_number);
            Ok(file_name)
        }
        Err(_) => {
            // TODO: should we return an errors here?
            error!("Failed upload QRP {} for vat {}", res.format, vat_number);
            Err(anyhow::anyhow!("Failed to upload QRP"))
        }
    }
}

#[get("/api/v1/healthz")]
async fn healthz() -> IoResult<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("application/json").finish())
}
