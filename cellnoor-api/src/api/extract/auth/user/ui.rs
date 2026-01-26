use std::collections::HashSet;

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
    pub fn into_authenticated_user(self) -> PrivateClaims {
        self.user
    }
}

impl FromEncodedJwt for User {}
