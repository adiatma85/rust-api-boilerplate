use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub jwt_secret: String,
}
