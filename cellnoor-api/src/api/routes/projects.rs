use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Extension,
    extract::{Path, Request},
    handler::Handler,
    middleware::Next,
    response::Response,
};
use cellnoor_models::IdParameter;
use tower::ServiceBuilder;

use crate::{
    api::{
        auth::{self, AuthenticatedUser},
        middleware::{admin_required, created_status_code},
    },
    state::AppState,
};
use add_person::add_person_to_project;
use create::create_project;
use index::index_projects;
// use index_chromium_datasets::index_chromium_datasets;
use index_people::index_project_people;
use index_specimens::index_project_specimens;
use remove_person::remove_person_from_project;
use show::show_project;

pub(super) mod add_person;
pub(super) mod create;
pub(super) mod index;
pub(super) mod index_chromium_datasets;
pub(super) mod index_people;
pub(super) mod index_specimens;
pub(super) mod remove_person;
pub(super) mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    let id_router = ApiRouter::new()
        .api_route("/", get(show_project))
        .api_route(
            "/people",
            get(index_project_people)
                .patch(add_person_to_project)
                .delete(
                    remove_person_from_project.layer(axum::middleware::from_fn(admin_required)),
                ),
        )
        .api_route("/specimens", get(index_project_specimens))
        .api_route("/chromium-datasets", get(async || ()))
        .layer(axum::middleware::from_fn(authorize_project_access));

    let creation_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(created_status_code));

    ApiRouter::new()
        .api_route(
            "/",
            post(create_project.layer(creation_middleware)).get(index_projects),
        )
        .nest("/{id}", id_router)
}

async fn authorize_project_access(
    Path(IdParameter { id: project_id }): Path<IdParameter>,
    Extension(user): Extension<AuthenticatedUser>,
    request: Request,
    next: Next,
) -> Result<Response, auth::Error> {
    if user.is_staff() {
        return Ok(next.run(request).await);
    }

    if !user.projects().contains(&project_id) {
        return Err(auth::Error::PermissionDenied);
    }

    Ok(next.run(request).await)
}
