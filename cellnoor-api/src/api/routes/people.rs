use crate::{
    api::middleware::{admin_required, created_status_code},
    state::AppState,
};
use aide::{
    axum::{
        ApiRouter,
        routing::{get, post_with},
    },
    transform::TransformOperation,
};
use axum::{Json, handler::Handler, http::StatusCode};
use create::create_person;
use index::index_people;
use index_projects::index_person_projects;
use index_specimens::index_person_specimens;
use show::show_person;
use tower::ServiceBuilder;
use update::update_person;

pub(super) mod create;
pub(super) mod index;
pub(super) mod index_chromium_datasets;
pub(super) mod index_projects;
pub(super) mod index_specimens;
pub(super) mod show;
pub(super) mod update;

pub(super) fn router() -> ApiRouter<AppState> {
    let post_and_patch_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(created_status_code));

    let person_items = ApiRouter::new()
        .api_route("/projects", get(index_person_projects))
        .api_route("/specimens", get(index_person_specimens))
        .api_route("/chromium-datasets", get(async || ()));

    let id_router = ApiRouter::new()
        .api_route(
            "/",
            get(show_person).patch_with(
                update_person.layer(post_and_patch_middleware.clone()),
                post_and_patch_docs,
            ),
        )
        .merge(person_items);

    ApiRouter::new()
        .api_route(
            "/",
            post_with(
                create_person.layer(post_and_patch_middleware),
                post_and_patch_docs,
            )
            .get(index_people),
        )
        .nest("/{id}", id_router)
}

fn post_and_patch_docs(api_docs: TransformOperation) -> TransformOperation {
    api_docs.response::<{ StatusCode::UNPROCESSABLE_ENTITY.as_u16() }, Json<create::Error>>()
}
