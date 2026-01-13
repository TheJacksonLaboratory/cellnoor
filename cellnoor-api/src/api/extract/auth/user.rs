mod api;
mod ui;

use std::sync::LazyLock;

pub use api::AuthenticatedUser;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
pub use ui::UiUser;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Deserialize)]
struct UserClaims {
    sub: Uuid,
    exp: usize,
    jti: Uuid,
}

impl UserClaims {
    fn from_jwt(encoded_jwt: &[u8], decoding_key: &DecodingKey) -> Result<Self, super::Error> {
        static VALIDATION: LazyLock<Validation> =
            LazyLock::new(|| Validation::new(Algorithm::HS512));

        let token = jsonwebtoken::decode(encoded_jwt, decoding_key, &VALIDATION).map_err(|e| {
            super::Error::Unauthorized {
                message: e.to_string(),
            }
        })?;

        Ok(token.claims)
    }
}
