use axum::extract::{FromRequest, FromRequestParts};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    api::{self, auth, request::Request},
    db,
    state::AppState,
};

#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(super::super::Error))]
pub struct Json<T>(pub T);

impl<Req, Resp> Request<Resp> for Json<Req>
where
    Req: Request<Resp>,
{
    type Authorized = Req::Authorized;
    type ValidationData = Req::ValidationData;

    async fn fetch_validation_data(
        &self,
        db_conn: db::DbConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        self.0.fetch_validation_data(db_conn).await
    }

    fn authorize(
        self,
        authorization_data: auth::AuthorizationData,
    ) -> Result<Self::Authorized, auth::Error> {
        self.0.authorize(authorization_data)
    }
}
