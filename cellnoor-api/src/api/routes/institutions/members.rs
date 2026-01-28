use aide::axum::{
    ApiRouter,
    routing::{get, get_with},
};
use cellnoor_models::person::{PersonQuery, PersonSummary};

use crate::{api::docs::db_and_auth_error_docs, state::AppState};

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/{id}/members",
        get_with(index::index_institution_members, db_and_auth_error_docs),
    )
}
