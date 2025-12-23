use axum::extract::FromRequestParts;

#[derive(FromRequestParts)]
#[from_request(via(serde_qs::axum::QsQuery), rejection(super::super::ErrorResponse))]
pub struct QsQuery<T>(pub T);
