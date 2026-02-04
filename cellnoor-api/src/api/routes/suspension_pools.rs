use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_suspension_pool;
use index::index_suspension_pools;
use show::show_suspension_pool;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod index;
pub mod measurements;
pub mod show;
pub mod suspensions;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_suspension_pool.layer(admin_required_creation!()))
                .get(index_suspension_pools),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_suspension_pool))
        .nest("/suspensions", suspensions::router())
        .nest("/measurements", measurements::router())
}
