use crate::db::DbConnection;
use crate::{db, state::AppState};
use aide::OperationIo;
use axum::Json;
use axum::response::IntoResponse;
use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::NewPerson;
use cellnoor_models::person::Person;
use cellnoor_schema::people;
use diesel_async::RunQueryDsl;
use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;
use std::sync::LazyLock;
use uuid::Uuid;

pub async fn create_person(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(person): Json<NewPerson>,
) -> Result<Json<Person>, Error> {
    validate_email(person.email())?;
    let id = insert_person(person, &mut db_conn).await?;

    Ok(super::show::select_person_by_id(id, &mut db_conn)
        .await
        .map(Json)?)
}

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

pub(super) fn validate_email(email: &str) -> Result<(), Error> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(Error::InvalidEmail);
    }

    Ok(())
}

pub async fn insert_person(person: NewPerson, db_conn: &mut DbConnection) -> Result<Uuid, Error> {
    Ok(diesel::insert_into(people::table)
        .values(person)
        .returning(people::id)
        .get_result(db_conn)
        .await?)
}

#[derive(Debug, thiserror::Error, Serialize, JsonSchema, OperationIo)]
#[serde(rename_all = "snake_case", tag = "type")]
#[schemars(rename = "CreatePersonError")]
#[error(transparent)]
pub enum Error {
    Database(#[from] db::Error),
    #[error("invalid email")]
    InvalidEmail,
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Self::Database(err.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Database(e) => e.into_response(),
            Self::InvalidEmail => (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response(),
        }
    }
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
