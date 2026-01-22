use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::{Person, PersonId, PersonUpdate};
use diesel::{
    RunQueryDsl,
    prelude::*,
    sql_types::{Array, Text},
};

use super::create::validate_email;
use crate::{
    api::{
        self,
        auth::{self, AuthorizationData},
        extract::{Json, auth::AuthenticatedUser},
        routes::{ApiResponse, handle_api_request},
    },
    db,
    state::AppState,
};

pub(super) async fn update_person(
    id: PersonId,
    state: State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<PersonUpdate>,
) -> ApiResponse<Person> {
    let item = handle_api_request(state, user, (id, request)).await?;
    Ok((StatusCode::OK, item))
}

define_sql_function! {fn grant_roles_to_user(user_id: Text, roles: Array<Text>)}

define_sql_function! {fn revoke_roles_from_user(user_id: Text, roles: Array<Text>)}

impl db::Operation<Person> for (PersonId, PersonUpdate) {
    type Authorized = Self;
    type ValidationData = Option<String>;

    async fn fetch_validation_data(
        &self,
        _db_conn: db::DbConnection,
    ) -> Result<Option<String>, db::Error> {
        let (_person_id, update_data) = &self;
        // Ideally we could return a reference but that doesn't work (unless I don't know what I'm doing)
        Ok(update_data.email().map(str::to_owned))
    }

    fn authorize(self, authorization_data: AuthorizationData) -> Result<Self, auth::Error> {
        if !authorization_data.is_admin() {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(self)
    }

    fn validate(_authorized_request: &Self, email: Option<String>) -> Result<(), api::DataError> {
        let Some(email) = email else {
            return Ok(());
        };

        Ok(validate_email(&email)?)
    }

    fn execute(
        (person_id, mut update): Self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Person, api::Error> {
        update.set_id(person_id.0);

        diesel::update(&update).set(&update).execute(db_conn)?;

        PersonId::execute(person_id, db_conn)
    }
}
