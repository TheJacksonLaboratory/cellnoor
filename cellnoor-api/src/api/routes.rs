use axum::{Json, Router, extract::State, http::StatusCode};
use axum_extra::routing::TypedPath;

use crate::{
    api::{error::ErrorResponse, extract::auth::AuthenticatedUser},
    db,
    state::AppState,
};

mod cdna;
mod chromium_datasets;
mod chromium_runs;
mod gem_pools;
mod institutions;
mod labs;
mod libraries;
mod multiplexing_tags;
mod people;
mod sequencing_runs;
mod specimens;
mod suspension_pools;
mod suspensions;
mod tenx_assays;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/institutions", institutions::router())
        .nest("/people", people::router())
        .nest("/labs", labs::router())
        .nest("/specimens", specimens::router())
        .nest("/10x-assays", tenx_assays::router())
        .nest("/sequencing-runs", sequencing_runs::router())
        .nest("/multiplexing-tags", multiplexing_tags::router())
        .nest("/suspensions", suspensions::router())
        .nest("/suspension-pools", suspension_pools::router())
        .nest("/chromium-runs", chromium_runs::router())
        .nest("/gem-pools", gem_pools::router())
        .nest("/cdna", cdna::router())
        .nest("/libraries", libraries::router())
        .nest("/chromium-datasets", chromium_datasets::router())
}

type ApiResponse<T> = Result<(StatusCode, Json<T>), super::error::ErrorResponse>;

#[derive(TypedPath)]
#[typed_path("/")]
struct Root;

async fn inner_handler<Request, Response>(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    request: Request,
) -> Result<Json<Response>, ErrorResponse>
where
    Request: std::fmt::Debug + db::Operation<Response> + Send + 'static,
    Response: Send + 'static,
{
    tracing::info!("{request:?}");

    let db_conn = state.db_conn().await?;

    db_conn
        .interact(move |db_conn| request.execute_as_user(user.id(), db_conn))
        .await?
        .map(Json)
        .map_err(ErrorResponse::from)
}
