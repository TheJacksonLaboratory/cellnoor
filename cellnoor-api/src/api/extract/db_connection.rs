use std::ops::Deref;

use axum::extract::FromRequestParts;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Object};

use crate::{api, db::DbConnection, state::AppState};

impl FromRequestParts<AppState> for DbConnection {
    type Rejection = api::Error;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, api::Error> {
        Ok(state.db_conn().await?)
    }
}
