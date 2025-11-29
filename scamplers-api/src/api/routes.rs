use axum::{Json, Router, extract::State, http::StatusCode};
use axum_extra::routing::TypedPath;

use crate::{
    api::{error::ErrorResponse, extract::auth::AuthenticatedUser},
    db,
    state::AppState,
};

mod chromium_datasets;
mod chromium_runs;
mod institutions;
mod labs;
mod multiplexing_tags;
mod nucleic_acids;
mod people;
mod sequencing_runs;
mod specimens;
mod suspensions;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/institutions", institutions::router())
        .nest("/people", people::router())
        .nest("/labs", labs::router())
        .nest("/specimens", specimens::router())
        .nest("/multiplexing-tags", multiplexing_tags::router())
        .nest("/suspensions", suspensions::router())
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

    let response = db_conn
        .interact(move |db_conn| request.execute_as_user(user.id(), db_conn))
        .await??;

    Ok(Json(response))
}
