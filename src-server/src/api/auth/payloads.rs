use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthenticateParams {
    pub code: String,
}