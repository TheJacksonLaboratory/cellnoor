use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::{Person, PersonCreation, PersonId};
use cellnoor_schema::people::dsl::{id, people};
use diesel::{
    RunQueryDsl,
    prelude::*,
    sql_types::{Array, Text},
};

use super::{ApiResponse, Root, handle_api_request};
use crate::{
    api::extract::{ValidJson, auth::AuthenticatedUser},
    db,
    state::AppState,
};

pub(super) async fn create_person(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<PersonCreation>,
) -> ApiResponse<Person> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

define_sql_function! {fn create_user_if_not_exists(user_id: Text, password: Text, roles: Array<Text>)}

impl db::Operation<Person> for PersonCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Person, db::Error> {
        // Get the ID of the inserted person first, then return the full `Person` struct
        let created_id: PersonId = diesel::insert_into(people)
            .values(&self)
            .returning(id)
            .get_result(db_conn)?;

        created_id.execute(db_conn)
    }
}
