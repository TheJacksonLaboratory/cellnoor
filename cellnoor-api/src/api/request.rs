use axum::{extract::State, http::StatusCode};
use diesel::{PgConnection, prelude::*};

use super::auth::{self, AuthenticatedUser, AuthorizationData};
use crate::db::{self, DbConnection};
use crate::state::AppState;

pub(super) const OK: u16 = StatusCode::OK.as_u16();
pub(super) const CREATED: u16 = StatusCode::CREATED.as_u16();

pub trait AuthorizedRequest<Resp> {
    type ValidationData;

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), super::DataError>;

    fn execute(self, db_conn: &mut PgConnection) -> Result<Resp, super::Error>;
}

pub trait Request<Resp>: Sized {
    type Authorized: AuthorizedRequest<Resp>;
    type ValidationData: Into<<Self::Authorized as AuthorizedRequest<Resp>>::ValidationData>;

    async fn fetch_validation_data(
        &self,
        db_conn: DbConnection,
    ) -> Result<Self::ValidationData, db::Error>;

    fn authorize(
        self,
        authorization_data: AuthorizationData,
    ) -> Result<Self::Authorized, auth::Error>;

    #[cfg(any(feature = "dummy-data", test))]
    fn execute_without_authorization(self, db_conn: &mut PgConnection) -> Resp {
        let authorized_request = self.authorize(AuthorizationData::new_admin()).unwrap();

        authorized_request.execute(db_conn).unwrap()
    }
}

type ApiResponse<T> = Result<(StatusCode, axum::Json<T>), super::Error>;

pub(super) async fn handle_api_request<Req, Resp, const SUCCESS_CODE: u16>(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    request: Req,
) -> ApiResponse<Resp>
where
    Req: std::fmt::Debug + Request<Resp>,
    Req::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    tracing::info!(request = ?request);

    let (db_conn1, db_conn2) = tokio::try_join!(state.db_conn(), state.db_conn())?;

    // Fetch the authorization data and validation data concurrently because speed™
    let (authorization_data, validation_data) = tokio::try_join!(
        user.authorization_data(db_conn1),
        request.fetch_validation_data(db_conn2)
    )?;

    let authorized_request = request.authorize(authorization_data)?;
    authorized_request.validate(validation_data.into())?;

    let db_conn = state.db_conn().await?;

    let response = db_conn
        .interact(|db_conn| db_conn.transaction(|tx| authorized_request.execute(tx)))
        .await?
        .map(axum::Json)?;

    Ok((StatusCode::from_u16(SUCCESS_CODE).unwrap(), response))
}
