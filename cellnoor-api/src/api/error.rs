use schemars::JsonSchema;
use serde::Serialize;

use crate::{api::auth, db};

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
pub enum ApiError {
    Auth(auth::Error),
    Database(db::Error),
}

#[derive(Serialize, JsonSchema)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}
