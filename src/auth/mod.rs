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
