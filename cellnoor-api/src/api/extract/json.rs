use axum::extract::{FromRequest, FromRequestParts, Request};
use serde::{Serialize, de::DeserializeOwned};

use crate::{api, state::AppState};

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(super::super::ErrorResponse))]
pub struct Json<T>(pub T);
