use std::collections::{HashSet, hash_set::Iter};

use cellnoor_schema::people;
use diesel::{pg::Pg, prelude::*};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::auth;

use super::FromEncodedJwt;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub(super) struct StandardClaims {
    pub(super) sub: Uuid,
    pub(super) iat: usize,
    pub(super) exp: usize,
    pub(super) iss: String,
    pub(super) aud: String,
}

impl FromEncodedJwt for StandardClaims {}

#[derive(Clone, Debug, Deserialize, HasQuery)]
#[diesel(table_name = people, check_for_backend(Pg))]
pub(super) struct UserFields {
    // (This comment should be read with imagined profanity and vitriol) better-auth, in its
    // definitely-not-vibe-coded glory, completely ignores my attempt to overwrite the `user.id` field, so I have to
    // use a custom field to get the user's ID from the JWT :)
    #[allow(dead_code)]
    #[diesel(column_name = id)]
    pub(super) user_id: Uuid,
    pub(super) is_admin: bool,
    pub(super) is_biology_staff: bool,
    pub(super) is_computational_staff: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Authorization {
    #[serde(flatten)]
    pub(super) user_fields: UserFields,
    pub(super) projects: HashSet<Uuid>,
}

impl Authorization {
    #[cfg(any(feature = "dummy-data", test))]
    pub fn new_admin() -> Self {
        Self {
            user_fields: UserFields {
                user_id: Uuid::nil(),
                is_admin: true,
                is_biology_staff: true,
                is_computational_staff: true,
            },
            projects: HashSet::new(),
        }
    }

    fn user_fields(&self) -> &UserFields {
        &self.user_fields
    }

    fn is_admin(&self) -> bool {
        self.user_fields().is_admin
    }

    fn is_biology_staff(&self) -> bool {
        self.user_fields().is_biology_staff
    }

    fn is_computational_staff(&self) -> bool {
        self.user_fields().is_computational_staff
    }

    fn is_staff(&self) -> bool {
        self.is_admin() || self.is_biology_staff() || self.is_computational_staff()
    }

    pub fn authorize_admin(&self) -> Result<(), auth::Error> {
        if !self.is_admin() {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(())
    }

    pub fn authorized_projects(
        self,
        requested_projects: Option<HashSet<Uuid>>,
    ) -> Option<HashSet<Uuid>> {
        let authorized_projects = self.projects;

        if self.is_staff() {
            return requested_projects;
        }

        let Some(projects) = requested_projects else {
            return Some(authorized_projects);
        };

        return Some(
            projects
                .into_iter()
                .filter(|p| authorized_projects.contains(p))
                .collect(),
        );
    }
}
