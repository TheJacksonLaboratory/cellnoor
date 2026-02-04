use aide::{
    axum::{
        ApiRouter,
        routing::{get, post_with},
    },
    transform::TransformOperation,
};
use axum::{Json, handler::Handler, http::StatusCode};
use create::create_person;
pub use create::validate_email;
use index::index_people;
use show::show_person;
use update::update_person;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod index;
pub mod projects;
pub mod show;
pub mod specimens;
pub mod update;

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
}

fn post_and_patch_docs(api_docs: TransformOperation) -> TransformOperation {
    api_docs.response::<{ StatusCode::UNPROCESSABLE_ENTITY.as_u16() }, Json<create::Error>>()
}
