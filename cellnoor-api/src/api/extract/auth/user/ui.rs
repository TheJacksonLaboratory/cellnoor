use serde::Deserialize;
use uuid::Uuid;

use super::FromEncodedJwt;
use super::common::*;

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    #[serde(flatten)]
    standard_claims: StandardClaims,
    user: PrivateClaims,
}

impl User {
    pub fn id(&self) -> Uuid {
        self.standard_claims.sub
    }

    pub(super) fn admin() -> Self {
        Self {
            standard_claims: StandardClaims {
                sub: Uuid::nil(),
                iat: usize::default(),
                exp: usize::default(),
                iss: String::default(),
                aud: String::default(),
            },
            user: PrivateClaims {
                user_fields: UserFields {
                    user_id: Uuid::nil(),
                    is_admin: true,
                    is_biology_staff: true,
                    is_computational_staff: true,
                },
                labs: Vec::new(),
            },
        }
    }

    fn user_fields(&self) -> &UserFields {
        &self.user.user_fields
    }

    pub fn is_admin(&self) -> bool {
        self.user_fields().is_admin
    }

    pub fn is_biology_staff(&self) -> bool {
        self.user_fields().is_biology_staff
    }

    pub fn is_computational_staff(&self) -> bool {
        self.user_fields().is_computational_staff
    }
}

impl FromEncodedJwt for User {}
