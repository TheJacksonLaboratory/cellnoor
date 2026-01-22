use axum::{Json, Router, extract::State, http::StatusCode};
use axum_extra::routing::TypedPath;
use diesel::Connection;

use crate::{
    api::{auth::AuthorizationData, error::ErrorResponse, extract::auth::AuthenticatedUser},
    db,
    state::AppState,
};

// pub(super) mod cdna;
// pub(super) mod chromium_datasets;
// pub(super) mod chromium_runs;
// pub(super) mod gem_pools;
pub(super) mod institutions;
// pub(super) mod libraries;
// pub(super) mod multiplexing_tags;
pub(super) mod people;
// pub(super) mod projects;
// pub(super) mod sequencing_runs;
// pub(super) mod specimens;
// pub(super) mod suspension_pools;
// pub(super) mod suspensions;
// pub(super) mod tenx_assays;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/institutions", institutions::router())
        .nest("/people", people::router())
    // .nest("/projects", projects::router())
    // .nest("/specimens", specimens::router())
    // .nest("/10x-assays", tenx_assays::router())
    // .nest("/sequencing-runs", sequencing_runs::router())
    // .nest("/multiplexing-tags", multiplexing_tags::router())
    // .nest("/suspensions", suspensions::router())
    // .nest("/suspension-pools", suspension_pools::router())
    // .nest("/chromium-runs", chromium_runs::router())
    // .nest("/gem-pools", gem_pools::router())
    // .nest("/cdna", cdna::router())
    // .nest("/libraries", libraries::router())
    // .nest("/chromium-datasets", chromium_datasets::router())
}

type ApiResponse<T> = Result<(StatusCode, Json<T>), super::error::ErrorResponse>;

#[derive(TypedPath)]
#[typed_path("/")]
struct Root;

async fn handle_api_request<Request, Response>(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    request: Request,
) -> Result<Json<Response>, ErrorResponse>
where
    Request: std::fmt::Debug + db::Operation<Response> + Send + 'static,
    Request::Authorized: Send,
    Response: Send + 'static,
{
    tracing::info!("{request:?}");

    let (db_conn1, db_conn2) = tokio::try_join!(state.db_conn(), state.db_conn())?;

    // Fetch the authorization data and validation data concurrently because speed™
    let (authorization_data, validation_data) = tokio::try_join!(
        user.authorization_data(db_conn1),
        request.fetch_validation_data(db_conn2)
    )?;

    let authorized_request = request.authorize(authorization_data)?;
    Request::validate(&authorized_request, validation_data)?;

    let db_conn = state.db_conn().await?;
    db_conn
        .interact(|db_conn| db_conn.transaction(|tx| Request::execute(authorized_request, tx)))
        .await?
        .map(Json)
        .map_err(ErrorResponse::from)
}
