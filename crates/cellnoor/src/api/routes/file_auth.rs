use axum::{
    Router,
    extract::{Request, State},
    middleware::Next,
    response::{Redirect, Response},
    routing::get,
};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::file_auth::{authorize_dataset_dir_access, authorize_project_dir_access},
    state::AppState,
};

pub fn router(state: AppState) -> Router<AppState> {
    let ok = get(async || ());
    // We mount the same handler twice: Once for paths like
    // file-auth/chromium-datasets/{id}/ and another for paths like
    // file-auth/chromium-datasets/{id}/file.html. Note that we put a trailing slash
    // for directories because caddy automatically does that
    Router::new()
        .route("/", ok.clone())
        // As long a user is authenticated, they can access top-level directories
        .route("/{dataset_type}", ok.clone())
        .route("/projects", ok)
        // Accessing a specific dataset triggers row-level security
        .route(
            "/{dataset_type}/{dataset_id}",
            get(authorize_dataset_dir_access),
        )
        .route(
            "/{dataset_type}/{dataset_id}/{*file_path}",
            get(authorize_dataset_dir_access),
        )
        // Accessing a specific project triggers row-level security
        .route(
            "/projects/{project_name}",
            get(authorize_project_dir_access),
        )
        .route(
            "/projects/{project_name}/{*file_path}",
            get(authorize_project_dir_access),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            redirect_unauthenticated_user,
        ))
}

async fn redirect_unauthenticated_user(
    State(state): State<AppState>,
    user: Result<AuthUser, Error>,
    request: Request,
    next: Next,
) -> Result<Response, Redirect> {
    if user.is_err() {
        // Since this route is meant for file authentication, we can confidently just
        // redirect to the file server after sign-in
        let redirect_to = format!(
            "{}?redirect_to={}{}",
            state.public_auth_url(),
            state.public_files_url(),
            request.uri().path()
        );

        tracing::debug!("redirecting user to: {redirect_to}");
        return Err(Redirect::to(&redirect_to));
    }

    Ok(next.run(request).await)
}
