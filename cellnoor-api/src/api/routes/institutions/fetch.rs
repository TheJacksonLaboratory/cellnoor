use axum::{extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, InstitutionId};
use cellnoor_schema::institutions::dsl::id;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db::{self},
    state::AppState,
};

pub(super) async fn fetch_institution(
    request: InstitutionId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Institution> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Institution> for InstitutionId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Institution, db::Error> {
        Ok(Institution::query().filter(id.eq(&self)).first(db_conn)?)
    }
}
