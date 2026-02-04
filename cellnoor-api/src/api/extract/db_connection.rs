use axum::extract::FromRequestParts;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

impl FromRequestParts<AppState> for DbConnection {
    type Rejection = db::Error;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, db::Error> {
        state.db_conn().await
    }
}
