use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_cdna;
pub(super) use create::{
    NucleicAcidParentInfo, gem_pools_to_library_specifications, validate_volume,
};
use index::index_cdna;
pub(super) use measurements::validate_electrophoretic_measurement;
use show::show_cdna;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod index;
pub mod measurements;
pub mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_cdna.layer(admin_required_creation!())).get(index_cdna),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_cdna))
        .nest("/measurements", measurements::router())
}
