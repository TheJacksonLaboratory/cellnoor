use axum::extract::State;
use cellnoor_models::suspension_pool::{SuspensionPool, SuspensionPoolId};
use cellnoor_schema::suspension_pools::id;
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

pub(super) async fn fetch_suspension_pool(
    suspension_id: SuspensionPoolId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<SuspensionPool> {
    let item = inner_handler(state, user, suspension_id).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<SuspensionPool> for SuspensionPoolId {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<SuspensionPool, db::Error> {
        Ok(SuspensionPool::query().filter(id.eq(self)).first(db_conn)?)
    }
}
