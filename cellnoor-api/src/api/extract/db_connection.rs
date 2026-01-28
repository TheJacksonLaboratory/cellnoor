use std::ops::Deref;

use axum::extract::FromRequestParts;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Object};

use crate::{
    api,
    db::{self, DbConnection},
    state::AppState,
};

impl FromRequestParts<AppState> for DbConnection {
    type Rejection = db::Error;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, db::Error> {
        Ok(state.db_conn().await?)
    }
}
