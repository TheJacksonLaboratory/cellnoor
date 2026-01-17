use serde::Deserialize;
use uuid::Uuid;

use crate::api::extract::auth::user::FromEncodedJwt;

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    sub: Uuid,
    is_admin: bool,
    is_biology_staff: bool,
    is_computational_staff: bool,
    iat: usize,
    exp: usize,
    iss: String,
    aud: String,
}

impl User {
    pub fn id(&self) -> Uuid {
        self.sub
    }

    pub(super) fn admin() -> Self {
        Self {
            sub: Uuid::nil(),
            is_admin: true,
            is_biology_staff: true,
            is_computational_staff: true,
            iat: usize::default(),
            exp: usize::default(),
            iss: String::default(),
            aud: String::default(),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn is_biology_staff(&self) -> bool {
        self.is_biology_staff
    }

    pub fn is_computational_staff(&self) -> bool {
        self.is_computational_staff
    }
}

impl FromEncodedJwt for User {}
