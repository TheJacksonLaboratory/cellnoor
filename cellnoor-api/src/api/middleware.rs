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
    api::auth::{self, AuthUser},
    state::AppState,
};

pub async fn authenticate_request(
    State(app_state): State<AppState>,
    cookies: CookieJar,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, auth::Error> {
    let user = AuthUser::from_request(&app_state, auth_header.as_ref(), &cookies).await?;

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

pub async fn admin_required(
    Extension(user): Extension<AuthUser>,
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

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! admin_required_creation {
    () => {
        tower::ServiceBuilder::new()
            .layer(axum::middleware::from_fn(
                crate::api::middleware::admin_required,
            ))
            .layer(axum::middleware::map_response(
                crate::api::middleware::created_status_code,
            ))
    };
}
