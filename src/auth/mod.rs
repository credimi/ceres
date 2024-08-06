use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Deserialize;

pub mod cerved_auth;

#[derive(Deserialize)]
struct CervedAuthRes {
    access_token: String,
    refresh_token: String,
    expires_in: u32,
}

#[derive(Deserialize, Default, Clone)]
struct CervedAuth {
    access_token: String,
    refresh_token: String,
    expires_in: u32,
    created_at: DateTime<Utc>,
}

impl From<CervedAuthRes> for CervedAuth {
    fn from(value: CervedAuthRes) -> Self {
        CervedAuth {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            expires_in: value.expires_in,
            created_at: Utc::now(),
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub struct CervedOAuthConfig {
    #[arg(long, env)]
    pub cerved_oauth_base_url: String,
    #[arg(long, env)]
    pub cerved_oauth_username: String,
    #[arg(long, env)]
    pub cerved_oauth_password: String,
}
