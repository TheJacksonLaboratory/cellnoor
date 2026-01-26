// Simple Axum middleware requires passing information from middleware to middleware (or middlware to handler) via the
// `axum::extract::Extension` struct. This isn't bad, but it implicitly means that one function relies on another to
// modify some kind of state. Instead, we use strongly-typed extractors, which are basically stateless. In theory, we
// could achieve this behavior by using tower's `Service` trait, but at that point you're basically reinventing Axum.
//
// All that said, the system laid out below satisfies the following requirements:
//  1. No extractor runs twice
//
//  2. Boilerplate is minimum - there's no reason to copy-paste getting a db connection, running the query in a
//  transaction, etc
//
//  3. A non-staff user can only see their projects (and the entities derived from them). For example, a user querying
//  for Chromium datasets should only see the Chromium datasets belonging to projects they also belong to. That means
//  the query itself needs to be modified. This system allows us a simple way to modify the query. This authorization
//  step transforms a type implementing `Request` into another type implementing `AuthorizedRequest`, so the developer
//  has to implement both to use the easy functions below.
use std::fmt::Debug;

use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, AsyncPgConnection};

use super::auth::{self, AuthenticatedUser};
use crate::api::extract::{Json, Path, PathAndJson, PathAndQuery, QsQuery};
use crate::db;
use crate::state::AppState;

pub trait AuthorizedRequest<Resp> {
    type ValidationData;

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), super::DataError>;

    fn handle(
        self,
        db_conn: &AsyncPgConnection,
    ) -> impl Future<Output = Result<Resp, super::Error>> + Send;
}

pub trait Request<Resp>: Sized {
    type Authorized: AuthorizedRequest<Resp> + Send;
    type ValidationData: Into<<Self::Authorized as AuthorizedRequest<Resp>>::ValidationData>;

    async fn fetch_validation_data(
        &self,
        db_conn: &AsyncPgConnection,
    ) -> Result<Self::ValidationData, db::Error>;

    fn authorize(self, user: AuthenticatedUser) -> Result<Self::Authorized, auth::Error>;

    #[cfg(any(feature = "dummy-data", test))]
    fn handle_without_authorization(
        self,
        db_conn: &AsyncPgConnection,
    ) -> impl Future<Output = Resp> + Send {
        let authorized_request = self.authorize(AuthenticatedUser::new_admin()).unwrap();

        async { authorized_request.handle(db_conn).await.unwrap() }
    }
}

type ApiResponse<T> = Result<(StatusCode, axum::Json<T>), super::Error>;

async fn handle_api_request<Req, Resp, const SUCCESS_CODE: u16>(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    request: Req,
) -> ApiResponse<Resp>
where
    Req: Debug + Request<Resp>,
    Req::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    tracing::info!(request = ?request);

    let mut db_conn = state.db_conn().await?;
    let validation_data = request.fetch_validation_data(&db_conn).await?;

    let authorized_request = request.authorize(user)?;
    authorized_request.validate(validation_data.into())?;

    let response = db_conn
        .transaction(|tx| authorized_request.handle(tx).scope_boxed())
        .await
        .map(axum::Json)?;

    Ok((StatusCode::from_u16(SUCCESS_CODE).unwrap(), response))
}

const OK: u16 = StatusCode::OK.as_u16();
const CREATED: u16 = StatusCode::CREATED.as_u16();

pub(super) async fn create<Data, Resp>(
    state: State<AppState>,
    user: AuthenticatedUser,
    request: Json<Data>,
) -> ApiResponse<Resp>
where
    Data: Debug + Request<Resp>,
    Data::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    handle_api_request::<_, _, CREATED>(state, user, request).await
}

pub(super) async fn show<Id, Resp>(
    state: State<AppState>,
    user: AuthenticatedUser,
    request: Path<Id>,
) -> ApiResponse<Resp>
where
    Id: Debug + Request<Resp>,
    Id::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    handle_api_request::<_, _, OK>(state, user, request).await
}

pub(super) async fn index<Query, Resp>(
    state: State<AppState>,
    user: AuthenticatedUser,
    request: QsQuery<Query>,
) -> ApiResponse<Resp>
where
    Query: Debug + Request<Resp>,
    Query::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    handle_api_request::<_, _, OK>(state, user, request).await
}

pub(super) async fn update<Id, Data, Resp>(
    state: State<AppState>,
    user: AuthenticatedUser,
    request: PathAndJson<Id, Data>,
) -> ApiResponse<Resp>
where
    Id: Debug,
    Data: Debug,
    (Id, Data): Request<Resp>,
    <(Id, Data) as Request<Resp>>::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    handle_api_request::<_, _, OK>(state, user, request).await
}

pub(super) async fn nested_create<Id, Data, Resp>(
    state: State<AppState>,
    user: AuthenticatedUser,
    request: PathAndJson<Id, Data>,
) -> ApiResponse<Resp>
where
    Id: Debug,
    Data: Debug,
    (Id, Data): Request<Resp>,
    <(Id, Data) as Request<Resp>>::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    handle_api_request::<_, _, CREATED>(state, user, request).await
}

pub(super) async fn nested_index<Id, Query, Resp>(
    state: State<AppState>,
    user: AuthenticatedUser,
    request: PathAndQuery<Id, Query>,
) -> ApiResponse<Resp>
where
    Id: Debug,
    Query: Debug,
    (Id, Query): Request<Resp>,
    <(Id, Query) as Request<Resp>>::Authorized: Send + 'static,
    Resp: Send + 'static,
{
    handle_api_request::<_, _, OK>(state, user, request).await
}
