use crate::{
    api::{
        docs::db_and_auth_error_docs,
        middleware::{admin_required, creation_status_code},
    },
    state::AppState,
};
use aide::{
    axum::{
        ApiRouter,
        routing::{get, get_with, post, post_with},
    },
    transform::TransformOperation,
};
use axum::{Json, handler::Handler, http::StatusCode};
use cellnoor_models::{
    person::{Person, PersonQuery, PersonSummary, PersonUpdate},
    specimen::{SpecimenQuery, SpecimenSummary},
};

pub use create::Error as CreatePersonError;
use tower::ServiceBuilder;

// mod chromium_datasets;
pub(super) mod create;
pub(super) mod index;
// pub(super) mod projects;
pub(super) mod show;
// pub(super) mod specimens;
pub(super) mod update;

pub(super) fn router() -> ApiRouter<AppState> {
    let post_and_patch_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(creation_status_code));

    let create_person = create::create_person.layer(post_and_patch_middleware.clone());
    let update_person = update::update_person.layer(post_and_patch_middleware);

    ApiRouter::new()
        .api_route(
            "/",
            post_with(create_person, post_and_patch_docs)
                .get_with(index::index_people, db_and_auth_error_docs),
        )
        .api_route(
            "/{id}",
            get_with(show::show_person, db_and_auth_error_docs)
                .patch_with(update_person, post_and_patch_docs),
        )
}

fn post_and_patch_docs(api_docs: TransformOperation) -> TransformOperation {
    // The function `db_and_auth_error_docs` maps `UNPROCESSABLE_ENTITY` to `db::Error`, but the error in this module already contains `db::Error`, so we just override it here
    api_docs
        .with(db_and_auth_error_docs)
        .response::<{ StatusCode::UNPROCESSABLE_ENTITY.as_u16() }, Json<create::Error>>()
}
