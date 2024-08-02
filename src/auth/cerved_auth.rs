use crate::auth::{CervedAuthRes, CervedOAuthConfig};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct CervedOAuthClient {
    cerved_base_url: String,
    cerved_oauth_config: CervedOAuthConfig,
    token: Arc<RwLock<CervedAuthRes>>,
}

impl CervedOAuthClient {
    pub async fn new(client: &reqwest::Client, base_url: &String, cerved_oauth_config: &CervedOAuthConfig) -> Self {
        CervedOAuthClient {
            cerved_base_url: base_url.clone(),
            cerved_oauth_config: cerved_oauth_config.clone(),
            token: Arc::new(RwLock::new(
                request_new_token(client, &base_url, &cerved_oauth_config)
                    .await
                    .expect("Cannot get Cerved OAuth token"),
            )),
        }
    }

    pub fn get_access_token(&self) -> String {
        let token = self.token.read().expect("Cannot acquire token in read");
        // TODO: check if token is expired and refresh
        token.access_token.clone()
    }
}

async fn request_new_token(
    http_client: &reqwest::Client,
    cerved_base_url: &String,
    cerved_oauth_config: &CervedOAuthConfig,
) -> anyhow::Result<CervedAuthRes> {
    // TODO: build from configuration
    let url = format!(
        "{}/cas/oauth/token?grant_type=password&client_id=cerved-client&username={}&password={}",
        cerved_base_url, cerved_oauth_config.username, cerved_oauth_config.password
    );
    Ok(http_client.get(url).send().await?.json::<CervedAuthRes>().await?)
}
