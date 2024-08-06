use crate::auth::{CervedAuth, CervedAuthRes, CervedOAuthConfig};
use crate::utils::logging::get_root_logger;
use slog::debug;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct CervedOAuthClient {
    cerved_oauth_config: CervedOAuthConfig,
    token: Arc<RwLock<CervedAuth>>,
    log: slog::Logger,
}

impl CervedOAuthClient {
    pub async fn new(client: &reqwest::Client, cerved_oauth_config: &CervedOAuthConfig) -> Self {
        CervedOAuthClient {
            cerved_oauth_config: cerved_oauth_config.clone(),
            token: Arc::new(RwLock::new(
                request_new_token(client, cerved_oauth_config)
                    .await
                    .expect("Unable to obtain Cerved OAuth token"),
            )),
            log: get_root_logger(),
        }
    }

    pub async fn get_access_token(&self, http_client: &reqwest::Client) -> anyhow::Result<String> {
        let token_read = self.token.read().expect("Cannot acquire read lock on token");
        if (token_read.created_at + chrono::Duration::seconds(token_read.expires_in as i64)) < chrono::Utc::now() {
            debug!(self.log, "Refreshing Cerved OAuth token...");
            let new_token = self
                .refresh_token(
                    http_client,
                    &token_read.refresh_token,
                    &self.cerved_oauth_config.cerved_oauth_base_url,
                )
                .await?;

            drop(token_read);
            let mut token_write = self.token.write().expect("Cannot acquire write lock on token");
            *token_write = new_token.clone();

            debug!(self.log, "Cerved OAuth token refreshed: {}", new_token.access_token);
            return Ok(new_token.access_token.clone());
        }
        Ok(token_read.access_token.clone())
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
        Ok(http_client.get(url).send().await?.json::<CervedAuthRes>().await?).and_then(|res| Ok(res.into()))
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
    Ok(http_client.get(url).send().await?.json::<CervedAuthRes>().await?).and_then(|res| Ok(res.into()))
}
