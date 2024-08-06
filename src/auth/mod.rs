use clap::Parser;
use serde::Deserialize;

pub mod cerved_auth;

#[derive(Deserialize, Default, Clone)]
struct CervedAuthRes {
    access_token: String,
    refresh_token: String,
    token_type: String,
    scope: String,
    expires_in: u32,
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
