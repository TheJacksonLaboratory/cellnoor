use axum::{extract::State, http::StatusCode};
use cellnoor_models::person::{Person, PersonId, PersonSummaryWithParents};
use cellnoor_schema::people::dsl::id;
use diesel::{PgConnection, prelude::*, sql_types::Text};

use super::{ApiResponse, inner_handler};
use crate::{api::extract::auth::AuthenticatedUser, db, state::AppState};

pub(super) async fn fetch_person(
    request: PersonId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Person> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Person> for PersonId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Person, db::Error> {
        diesel::define_sql_function! {fn get_user_roles(user_id: Text) -> Array<Text>}

        let info = PersonSummaryWithParents::query()
            .filter(id.eq(&self))
            .first(db_conn)?;

        let roles = diesel::select(get_user_roles(self.to_id_string())).get_result(db_conn)?;

        Ok(Person::new(info, roles))
    }
}
