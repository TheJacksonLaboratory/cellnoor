use serde::Deserialize;
use uuid::Uuid;

use crate::api::extract::auth::user::FromEncodedJwt;

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    sub: Uuid,
    jti: Uuid,
    iat: usize,
    exp: usize,
    iss: String,
    aud: String,
}

impl User {
    pub fn id(&self) -> Uuid {
        self.sub
    }
}

impl FromEncodedJwt for User {}
