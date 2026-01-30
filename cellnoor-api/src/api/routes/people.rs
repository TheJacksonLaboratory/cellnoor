use crate::{admin_required_creation, state::AppState};
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
use show::show_person;
use update::update_person;

pub(super) mod chromium_datasets;
pub(super) mod create;
pub(super) mod index;
pub(super) mod projects;
pub(super) mod show;
pub(super) mod specimens;
pub(super) mod update;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post_with(
                create_person.layer(admin_required_creation!()),
                post_and_patch_docs,
            )
            .get(index_people),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get(show_person).patch_with(
                update_person.layer(admin_required_creation!()),
                post_and_patch_docs,
            ),
        )
        .nest("/projects", projects::router())
        .nest("/specimens", specimens::router())
        .nest("/chromium-datasets", chromium_datasets::router())
}

fn post_and_patch_docs(api_docs: TransformOperation) -> TransformOperation {
    api_docs.response::<{ StatusCode::UNPROCESSABLE_ENTITY.as_u16() }, Json<create::Error>>()
}
