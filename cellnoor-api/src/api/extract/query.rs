use axum::extract::FromRequestParts;
use diesel_async::AsyncPgConnection;

use crate::{
    api::{auth, request::Request},
    db,
};

#[derive(Debug, FromRequestParts)]
#[from_request(via(serde_qs::axum::QsQuery), rejection(super::super::Error))]
pub struct QsQuery<T>(pub T);

impl<Req, Resp> Request<Resp> for QsQuery<Req>
where
    Req: Request<Resp>,
{
    type Authorized = Req::Authorized;
    type ValidationData = Req::ValidationData;

    async fn fetch_validation_data(
        &self,
        db_conn: &AsyncPgConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        self.0.fetch_validation_data(db_conn).await
    }

    fn authorize(self, user: auth::AuthenticatedUser) -> Result<Self::Authorized, auth::Error> {
        self.0.authorize(user)
    }
}
