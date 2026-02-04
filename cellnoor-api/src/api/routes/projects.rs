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
use create::create_project;
use index::index_projects;
use show::show_project;

use crate::{
    admin_required_creation,
    api::auth::{self, AuthUser},
    state::AppState,
};

pub mod chromium_datasets;
pub mod create;
pub mod index;
pub mod people;
pub mod show;
pub mod specimens;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_project.layer(admin_required_creation!())).get(index_projects),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_project))
        .nest("/people", people::router())
        .nest("/specimens", specimens::router())
        .nest("/chromium-datasets", chromium_datasets::router())
        .layer(axum::middleware::from_fn(authorize_project_access))
}

async fn authorize_project_access(
    Path(IdParameter { id: project_id }): Path<IdParameter>,
    Extension(user): Extension<AuthUser>,
    request: Request,
    next: Next,
) -> Result<Response, auth::Error> {
    if !user.has_access_to_project(&project_id) {
        return Err(auth::Error::PermissionDenied);
    }

    Ok(next.run(request).await)
}
