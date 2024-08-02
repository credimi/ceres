use crate::auth::cerved_auth::CervedOAuthClient;
use crate::qrp::{DeliveryStatus, QrpFormat, QrpRequest, QrpResponse};
use anyhow::anyhow;
use backon::{ExponentialBuilder, Retryable};

#[derive(Clone)]
pub struct CervedQrpClient {
    http_client: reqwest::Client,
    oauth_client: CervedOAuthClient,
}

impl CervedQrpClient {
    pub async fn new(http_client: reqwest::Client) -> Self {
        CervedQrpClient {
            http_client: http_client.clone(),
            oauth_client: CervedOAuthClient::new(&http_client).await,
        }
    }

    /// Generates the QRP. Retries the call when the response is in status "deferred"
    pub async fn generate_qrp_with_retry(&self, qrp_request: &QrpRequest) -> anyhow::Result<QrpResponse> {
        let _token = self.oauth_client.get_access_token();
        let token = _token.clone();
        let qrp_response = self
            .http_client
            // TODO: move into configuration
            .post("http://localhost:3001/cervedApiB2B/v1/purchase")
            .bearer_auth(token)
            .json(qrp_request)
            .send()
            .await?
            .json::<QrpResponse>()
            .await?;

        match qrp_response.delivery_status {
            DeliveryStatus::OK => Ok(qrp_response),
            DeliveryStatus::DEFERRED => {
                let token = _token;
                let request_id = qrp_response.request_id;
                let format = qrp_response.format;
                let to_retry = || async { self.read_qrp(&token, request_id, &format).await };
                Ok(to_retry
                    .retry(&ExponentialBuilder::default().with_max_times(10))
                    .when(|err| err.to_string() == "deferred")
                    .await?)
            }
        }
    }

    /// Read the QRP with request_id in the specified format. Retries the call when the response is in status "deferred"
    pub async fn read_qrp_with_retry(&self, request_id: u32, format: &QrpFormat) -> anyhow::Result<QrpResponse> {
        let token = self.oauth_client.get_access_token();
        let to_retry = || async { self.read_qrp(&token, request_id, &format).await };
        Ok(to_retry
            .retry(&ExponentialBuilder::default().with_max_times(10))
            .when(|err| err.to_string() == "deferred")
            .await?)
    }

    /// Read the QRP with request_id in the specified format. If the response status is "deferred", returns an error so that the call can be retried
    async fn read_qrp(&self, token: &String, request_id: u32, format: &QrpFormat) -> anyhow::Result<QrpResponse> {
        let res = self
            .http_client
            .get(format!(
                // TODO: move into configuration
                "http://localhost:3001/cervedApiB2B/v1/purchase/request/{}/format/{}",
                request_id, format
            ))
            .bearer_auth(token)
            .send()
            .await?
            .json::<QrpResponse>()
            .await?;

        match res.delivery_status {
            DeliveryStatus::OK => Ok(res),
            DeliveryStatus::DEFERRED => Err(anyhow!("deferred")),
        }
    }
}