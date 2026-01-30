use std::sync::Arc;

use axum::{
    Extension,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{TypedHeader, extract::CookieJar};
use headers::{Authorization, authorization::Bearer};

use crate::{
    api::auth::{self, AuthenticatedUser},
    state::AppState,
};

pub async fn authenticate_request(
    State(app_state): State<AppState>,
    cookies: CookieJar,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, auth::Error> {
    let user = AuthenticatedUser::from_request(&app_state, auth_header.as_ref(), &cookies).await?;

    // Wrap
    request.extensions_mut().insert(Arc::new(user));

    Ok(next.run(request).await)
}

pub async fn admin_required(
    Extension(user): Extension<Arc<AuthenticatedUser>>,
    request: Request,
    next: Next,
) -> Result<Response, auth::Error> {
    if !user.is_admin() {
        Err(auth::Error::PermissionDenied)?;
    }

    Ok(next.run(request).await)
}

pub async fn created_status_code(mut response: Response) -> Response {
    let status_code = response.status_mut();

    if status_code.is_success() {
        *status_code = StatusCode::CREATED;
    }

    response
}
