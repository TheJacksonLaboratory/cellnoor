use axum::{extract::State, http::StatusCode};
use cellnoor_models::person::{Person, PersonId, PersonUpdate};
use diesel::{
    RunQueryDsl,
    prelude::*,
    sql_types::{Array, Text},
};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn update_person(
    id: PersonId,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<PersonUpdate>,
) -> ApiResponse<Person> {
    let item = inner_handler(state, user, (id, request)).await?;
    Ok((StatusCode::OK, item))
}

define_sql_function! {fn grant_roles_to_user(user_id: Text, roles: Array<Text>)}

define_sql_function! {fn revoke_roles_from_user(user_id: Text, roles: Array<Text>)}

impl db::Operation<Person> for (PersonId, PersonUpdate) {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Person, db::Error> {
        let (id, mut update) = self;
        update.set_id(id.0);

        diesel::update(&update).set(&update).execute(db_conn)?;

        let id_string = id.to_id_string();

        if let Some(grant_roles) = update.grant_roles() {
            diesel::select(grant_roles_to_user(&id_string, grant_roles)).execute(db_conn)?;
        }

        if let Some(revoke_roles) = update.revoke_roles() {
            diesel::select(revoke_roles_from_user(&id_string, revoke_roles)).execute(db_conn)?;
        }

        id.execute(db_conn)
    }
}
