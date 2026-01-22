use crate::api;
use crate::api::request::Request;
use axum::RequestExt;
use axum::extract::FromRequest;
use serde::de::DeserializeOwned;

use super::json::Json;
use super::path::Path;

#[derive(Debug)]
pub struct PathAndJson<P, J>((P, J));

impl<S, P, J> FromRequest<S> for PathAndJson<P, J>
where
    S: Send + Sync,
    P: Send + DeserializeOwned + 'static,
    J: DeserializeOwned + 'static,
{
    type Rejection = api::Error;

    async fn from_request(
        mut body: axum::extract::Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Path(p) = body.extract_parts_with_state(state).await?;
        let Json(j) = body.extract_with_state(state).await?;

        Ok(Self((p, j)))
    }
}

impl<P, J, Resp> Request<Resp> for PathAndJson<P, J>
where
    (P, J): Request<Resp>,
{
    type Authorized = <(P, J) as Request<Resp>>::Authorized;
    type ValidationData = <(P, J) as Request<Resp>>::ValidationData;

    async fn fetch_validation_data(
        &self,
        db_conn: crate::db::DbConnection,
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
