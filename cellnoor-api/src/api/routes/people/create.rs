use super::{ApiResponse, Root, handle_api_request};
use crate::api;
use crate::api::auth::{self, AuthorizationData};
use crate::api::extract::Json;
use crate::db::Operation;
use crate::{api::extract::auth::AuthenticatedUser, db, state::AppState};
use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::PersonUpdate;
use cellnoor_models::person::{Person, PersonCreation, PersonId};
use cellnoor_schema::people::dsl::{id, people};
use diesel::{
    RunQueryDsl,
    prelude::*,
    sql_types::{Array, Text},
};
use regex::Regex;
use std::sync::LazyLock;

pub(super) async fn create_person(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<PersonCreation>,
) -> ApiResponse<Person> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Person> for PersonCreation {
    type Authorized = Self;
    type ValidationData = String;

    async fn fetch_validation_data(&self, _db_conn: db::DbConnection) -> Result<String, db::Error> {
        // Ideally we could return a reference but that doesn't work (unless I don't know what I'm doing)
        Ok(self.email().to_owned())
    }

    fn authorize(self, authorization_data: AuthorizationData) -> Result<Self, auth::Error> {
        if !authorization_data.is_admin() {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(self)
    }

    fn validate(_authorized_request: &Self, email: String) -> Result<(), api::DataError> {
        Ok(validate_email(&email)?)
    }

    fn execute(
        data: PersonCreation,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Person, api::Error> {
        // Get the ID of the inserted person first, then return the full `Person` struct
        let created_id: PersonId = diesel::insert_into(people)
            .values(data)
            .returning(id)
            .get_result(db_conn)?;

        PersonId::execute(created_id, db_conn)
    }
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "PersonValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error("{email} invalid: {message}")]
pub enum Error {
    Email { email: String, message: String },
}

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

pub(super) fn validate_email(email: &str) -> Result<(), Error> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(Error::Email {
            email: email.to_owned(),
            message: "invalid email".to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::EMAIL_REGEX;

    #[rstest]
    fn valid_email() {
        assert!(EMAIL_REGEX.is_match("peter.parker@spiderman.avengers"))
    }

    #[rstest]
    fn email_has_no_domain() {
        assert!(!EMAIL_REGEX.is_match("SpongeBob"))
    }

    #[rstest]
    fn email_contains_space() {
        assert!(!EMAIL_REGEX.is_match("Harry Potter"))
    }
}
