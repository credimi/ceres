use crate::auth::CervedAuthRes;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct CervedOAuthClient {
    token: Arc<RwLock<CervedAuthRes>>,
}

impl CervedOAuthClient {
    pub async fn new(client: &reqwest::Client) -> Self {
        CervedOAuthClient {
            token: Arc::new(RwLock::new(
                request_new_token(client).await.expect("Cannot get Cerved OAuth token"),
            )),
        }
    }

    pub fn get_access_token(&self) -> String {
        let token = self.token.read().expect("Cannot acquire token in read");
        // TODO: check if token is expired and refresh
        token.access_token.clone()
    }
}

async fn request_new_token(http_client: &reqwest::Client) -> anyhow::Result<CervedAuthRes> {
    // TODO: build from configuration
    let url = "http://localhost:3001/cas/oauth/token?grant_type=password&client_id=cerved-client&username=username&password=password";
    Ok(http_client.get(url).send().await?.json::<CervedAuthRes>().await?)
}
