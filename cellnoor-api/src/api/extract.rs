use crate::{api::request::Request, db};
use axum::extract::FromRequest;
pub use json::Json;
pub use path::Path;
pub use path_and_json::PathAndJson;
pub use path_and_query::PathAndQuery;
pub use query::QsQuery;

pub mod auth;
mod json;
mod path;
mod path_and_json;
mod path_and_query;
mod query;

// impl<P, Q, Resp> Request<Resp> for (path::Path<P>, query::QsQuery<Q>)
// where
//     (P, Q): Request<Resp>,
// {
//     type Authorized = <(P, Q) as Request<Resp>>::Authorized;
//     type ValidationData = <(P, Q) as Request<Resp>>::ValidationData;

//     async fn fetch_validation_data(
//         &self,
//         db_conn: db::DbConnection,
//     ) -> Result<Self::ValidationData, db::Error> {
//         let (path::Path(p), query::QsQuery(q)) = self;
//         (p, q).fetch_validation_data(db_conn).await
//     }

//     fn authorize(
//         self,
//         authorization: auth::AuthorizationData,
//     ) -> Result<Self::Authorized, auth::Error> {
//         self.0.authorize(authorization)
//     }
// }
