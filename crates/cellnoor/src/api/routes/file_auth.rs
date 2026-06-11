use aide::axum::{ApiRouter, routing::get};

use crate::{
    handlers::{
        file_auth::{authorize_dataset_dir_access, authorize_project_dir_access},
        redirect_unauthenticated_user,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    // We mount the same handler twice: Once for paths like
    // file-auth/chromium-datasets/{id}/ and another for paths like
    // file-auth/chromium-datasets/{id}/file.html. Note that we put a trailing slash
    // for directories because caddy automatically does that
    ApiRouter::new()
        // As long a user is authenticated, they can access top-level directories.
        .api_route("/{dataset_type}/", get(redirect_unauthenticated_user))
        .api_route("/projects/", get(redirect_unauthenticated_user))
        // Accessing a specific dataset triggers row-level security
        .api_route(
            "/{dataset_type}/{dataset_id}/",
            get(authorize_dataset_dir_access),
        )
        .api_route(
            "/{dataset_type}/{dataset_id}/{*file_path}",
            get(authorize_dataset_dir_access),
        )
        // Accessing a specific project triggers row-level security
        .api_route(
            "/projects/{project_name}/",
            get(authorize_project_dir_access),
        )
        .api_route(
            "/projects/{project_name}/{*file_path}",
            get(authorize_project_dir_access),
        )
}
