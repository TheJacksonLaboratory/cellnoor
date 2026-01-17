use axum::{extract::State, http::StatusCode};
use cellnoor_models::chromium_run::{GemPool, GemPoolId};
use cellnoor_schema::gem_pools;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, handle_api_request},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_gem_pool(
    request: GemPoolId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<GemPool> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<GemPool> for GemPoolId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<GemPool, db::Error> {
        Ok(GemPool::query()
            .filter(gem_pools::id.eq(self))
            .first(db_conn)?)
    }
}
