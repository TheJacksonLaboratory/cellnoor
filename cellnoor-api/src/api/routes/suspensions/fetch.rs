use axum::extract::State;
use cellnoor_models::suspension::{Suspension, SuspensionId};
use cellnoor_schema::suspensions::id;
use diesel::prelude::*;
use reqwest::StatusCode;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_suspension(
    suspension_id: SuspensionId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Suspension> {
    let item = inner_handler(state, user, suspension_id).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Suspension> for SuspensionId {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Suspension, db::Error> {
        Ok(Suspension::query().filter(id.eq(self)).first(db_conn)?)
    }
}
