use aide::axum::{ApiRouter, routing::get};

use crate::{
    api::routes::institutions::members::index::index_institution_members, state::AppState,
};

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_institution_members))
}
