use std::collections::HashSet;

use serde::Deserialize;
use uuid::Uuid;

use super::FromEncodedJwt;
use super::{AuthenticatedUserInner, common::*};

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    #[allow(dead_code)]
    #[serde(flatten)]
    standard_claims: StandardClaims,
    user: PrivateClaims,
    projects: HashSet<Uuid>,
}

impl User {
    pub fn into_authenticated_user(self) -> AuthenticatedUserInner {
        let Self { user, projects, .. } = self;

        AuthenticatedUserInner { user, projects }
    }
}

impl FromEncodedJwt for User {}
