use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::{Person, PersonId};
use cellnoor_schema::people::dsl::id;
use diesel::{PgConnection, prelude::*};

use super::{ApiResponse, handle_api_request};
use crate::{api::extract::auth::AuthenticatedUser, db, state::AppState};

pub(super) async fn fetch_person(
    request: PersonId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Person> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Person> for PersonId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Person, db::Error> {
        Ok(Person::query().filter(id.eq(&self)).first(db_conn)?)
    }
}
