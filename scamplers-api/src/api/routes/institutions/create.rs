use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use scamplers_models::institution::{Creation, Institution};
use scamplers_schema::institutions::dsl::institutions;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_institution(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<Creation>,
) -> ApiResponse<Institution> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Institution> for Creation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Institution, db::Error> {
        Ok(diesel::insert_into(institutions)
            .values(self)
            .returning(Institution::as_returning())
            .get_result(db_conn)?)
    }
}
