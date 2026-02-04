use aide::axum::{ApiRouter, routing::post};

use crate::{
    admin_required_creation,
    api::routes::sequencing_runs::libraries::add_to_sequencing_run::add_libraries_to_sequencing_run,
    state::AppState,
};

pub mod create;
pub mod libraries;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create::create_sequencing_run))
        .nest("/{id}", id_router())
        .layer(admin_required_creation!())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/libraries", post(add_libraries_to_sequencing_run))
}
