mod error;
mod user;

use std::sync::RwLockReadGuard;

use axum::{
    Extension, debug_middleware,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, cookie::Cookie},
};
pub use error::Error;
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{TokenData, Validation};
pub use user::AuthenticatedUser;

use crate::{
    api::{self, auth},
    state::{AppState, JwtDecodingKey},
};

#[debug_middleware]
pub async fn authenticate_request(
    State(app_state): State<AppState>,
    cookies: CookieJar,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, api::Error> {
    let user = AuthenticatedUser::from_request(&app_state, auth_header.as_ref(), &cookies).await?;

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

#[debug_middleware]
pub async fn admin_required(
    Extension(user): Extension<AuthenticatedUser>,
    request: Request,
    next: Next,
) -> Result<Response, api::Error> {
    if !user.is_admin() {
        Err(auth::Error::PermissionDenied)?
    }

    Ok(next.run(request).await)
}
