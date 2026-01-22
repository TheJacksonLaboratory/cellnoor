use std::collections::HashSet;

use serde::Deserialize;
use uuid::Uuid;

use super::FromEncodedJwt;
use super::common::*;

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    #[serde(flatten)]
    standard_claims: StandardClaims,
    user: AuthorizationData,
}

impl User {
    pub fn into_authorization_data(self) -> AuthorizationData {
        self.user
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
            user: AuthorizationData {
                user_fields: UserFields {
                    user_id: Uuid::nil(),
                    is_admin: true,
                    is_biology_staff: true,
                    is_computational_staff: true,
                },
                projects: HashSet::new(),
            },
        }
    }
}

impl FromEncodedJwt for User {}
