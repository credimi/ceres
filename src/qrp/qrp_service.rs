use crate::aws::aws_s3::S3Client;
use crate::errors::ErrorKind::{CervedError, S3Error};
use crate::qrp::cerved_qrp::{CervedQrpClient, QrpOrDeferred};
use crate::qrp::{QrpFormat, QrpRequest, QrpResponse};
use actix_web::HttpResponse;
use chrono::Utc;
use serde_json::json;
use std::error::Error;
use tracing::{error, info};

pub async fn request_qrp(
    qrp_client: CervedQrpClient,
    s3_client: S3Client,
    vat_number: String,
    user: String,
    qrp_req: &QrpRequest,
) -> Result<HttpResponse, Box<dyn Error>> {
    let result = qrp_client.generate_qrp(qrp_req).await;

    Ok(if let Ok(response) = result {
        match response {
            QrpOrDeferred::Qrp(res) => get_and_upload(qrp_client, s3_client, &vat_number, &user, &res).await?,
            QrpOrDeferred::Deferred(deferred_res) => {
                // TODO: save the request on db and try later
                tokio::spawn(async move {
                    let res = qrp_client
                        .read_qrp_with_retry(deferred_res.request_id, deferred_res.format)
                        .await
                        .expect("Failed to generate QRP");
                    get_and_upload(qrp_client, s3_client, &vat_number, &user, &res)
                        .await
                        .expect("Failed to upload QRP");
                });
                HttpResponse::Accepted().json(json!({"message": "QRP generation requested"}))
            }
        }
    } else {
        error!("Failed to generate QRP for vat {}: {:?}", vat_number, &result.err());
        CervedError.error_response()
    })
}

async fn get_and_upload(
    qrp_client: CervedQrpClient,
    s3_client: S3Client,
    vat_number: &String,
    user: &String,
    res: &QrpResponse,
) -> Result<HttpResponse, Box<dyn Error>> {
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
