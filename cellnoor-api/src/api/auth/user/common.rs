use cellnoor_schema::people;
use diesel::{pg::Pg, prelude::*};
use serde::Deserialize;
use uuid::Uuid;

use super::FromEncodedJwt;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub(super) struct StandardClaims {
    pub(super) jti: Uuid,
    pub(super) sub: Uuid,
    pub(super) iat: usize,
    pub(super) exp: usize,
    pub(super) iss: String,
    pub(super) aud: String,
}

impl FromEncodedJwt for StandardClaims {}

#[derive(Clone, Debug, Deserialize, Selectable, Queryable)]
#[diesel(table_name = people, check_for_backend(Pg))]
pub(super) struct PrivateClaims {
    // (This comment should be read with imagined profanity and vitriol) better-auth, in its
    // definitely-not-vibe-coded glory, completely ignores my attempt to overwrite the `user.id`
    // field, so I have to use a custom field to get the user's ID from the JWT :)
    #[allow(dead_code)]
    #[diesel(column_name = id)]
    pub(super) user_id: Uuid,
    pub(super) is_admin: bool,
    pub(super) is_biology_staff: bool,
    pub(super) is_computational_staff: bool,
}
