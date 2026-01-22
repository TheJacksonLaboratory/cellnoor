use axum::extract::FromRequestParts;

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(super::super::ErrorResponse))]
pub struct Path<T>(pub T);
