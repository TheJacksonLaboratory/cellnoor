use crate::api;
use crate::api::request::Request;
use axum::extract::{FromRequest, FromRequestParts};
use axum::{RequestExt, RequestPartsExt};
use diesel_async::AsyncPgConnection;
use serde::de::DeserializeOwned;

use super::path::Path;
use super::query::QsQuery;

#[derive(Debug)]
pub struct PathAndQuery<P, Q>((P, Q));

impl<S, P, J> FromRequestParts<S> for PathAndQuery<P, J>
where
    S: Send + Sync,
    P: Send + DeserializeOwned + 'static,
    J: DeserializeOwned + 'static,
{
    type Rejection = api::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Path(p) = parts.extract_with_state(state).await?;
        let QsQuery(q) = parts.extract_with_state(state).await?;

        Ok(Self((p, q)))
    }
}

impl<P, Q, Resp> Request<Resp> for PathAndQuery<P, Q>
where
    (P, Q): Request<Resp>,
{
    type Authorized = <(P, Q) as Request<Resp>>::Authorized;
    type ValidationData = <(P, Q) as Request<Resp>>::ValidationData;

    async fn fetch_validation_data(
        &self,
        db_conn: &AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        self.0.fetch_validation_data(db_conn).await
    }

    fn authorize(
        self,
        authorization_data: api::auth::AuthorizationData,
    ) -> Result<Self::Authorized, api::auth::Error> {
        self.0.authorize(authorization_data)
    }
}
