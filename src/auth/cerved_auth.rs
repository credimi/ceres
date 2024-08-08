use crate::auth::{CervedAuth, CervedAuthRes, CervedOAuthConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Clone)]
pub struct CervedOAuthClient {
    cerved_oauth_config: CervedOAuthConfig,
    token: Arc<Mutex<CervedAuth>>,
}

impl CervedOAuthClient {
    pub async fn new(client: &reqwest::Client, cerved_oauth_config: &CervedOAuthConfig) -> Self {
        CervedOAuthClient {
            cerved_oauth_config: cerved_oauth_config.clone(),
            token: Arc::new(Mutex::new(
                request_new_token(client, cerved_oauth_config)
                    .await
                    .expect("Unable to obtain Cerved OAuth token"),
            )),
        }
    }

    pub async fn get_access_token(&self, http_client: &reqwest::Client) -> anyhow::Result<String> {
        let mut token_guard = self.token.lock().await;
        if (token_guard.created_at + chrono::Duration::seconds(token_guard.expires_in as i64)) < chrono::Utc::now() {
            debug!("Refreshing Cerved OAuth token...");
            let new_token = self
                .refresh_token(
                    http_client,
                    &token_guard.refresh_token,
                    &self.cerved_oauth_config.cerved_oauth_base_url,
                )
                .await?;
            *token_guard = new_token.clone();

            debug!("Cerved OAuth token refreshed: {}", new_token.access_token);
            return Ok(new_token.access_token);
        }
        Ok(token_guard.access_token.clone())
    }

    async fn refresh_token(
        &self,
        http_client: &reqwest::Client,
        refresh_token: &String,
        oauth_base_url: &String,
    ) -> anyhow::Result<CervedAuth> {
        let url = format!(
            "{}/cas/oauth/token?grant_type=refresh_token&client_id=cerved-client&refresh_token={}",
            oauth_base_url, refresh_token
        );
        Ok(http_client.get(url).send().await?.json::<CervedAuthRes>().await?).map(|res| res.into())
    }
}

async fn request_new_token(
    http_client: &reqwest::Client,
    cerved_oauth_config: &CervedOAuthConfig,
) -> anyhow::Result<CervedAuth> {
    let url = format!(
        "{}/cas/oauth/token?grant_type=password&client_id=cerved-client&username={}&password={}",
        cerved_oauth_config.cerved_oauth_base_url,
        cerved_oauth_config.cerved_oauth_username,
        cerved_oauth_config.cerved_oauth_password
    );
    Ok(http_client.get(url).send().await?.json::<CervedAuthRes>().await?).map(|res| res.into())
}
