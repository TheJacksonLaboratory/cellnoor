use axum::{Router, http::StatusCode};
use axum_extra::routing::{Resource, RouterExt};
use cellnoor_models::{
    institution::{Institution, InstitutionCreation, InstitutionId, InstitutionQuery},
    person::{PersonQuery, PersonSummary},
};
use deadpool_diesel::Status;

use crate::{
    api::{
        extract::{Json, Path, PathAndQuery, QsQuery},
        request::{CREATED, OK, handle_api_request},
    },
    state::AppState,
};

mod create;
mod fetch;
mod list;
mod members;

pub(super) fn router() -> Router<AppState> {
    let resource = "institutions";
    let router: Router<AppState> = Resource::named(resource)
        .create(handle_api_request::<Json<InstitutionCreation>, Institution, CREATED>)
        .show(handle_api_request::<Path<InstitutionId>, Institution, OK>)
        .index(handle_api_request::<QsQuery<InstitutionQuery>, Vec<Institution>, OK>)
        .into();

    router.merge(
        Resource::named(&format!("{resource}/{{id}}/members")).index(
            handle_api_request::<PathAndQuery<InstitutionId, PersonQuery>, Vec<PersonSummary>, OK>,
        ),
    )
}
