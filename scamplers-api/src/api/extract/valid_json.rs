use axum::{Json, extract::FromRequest};
use serde::{Serialize, de::DeserializeOwned};

use crate::{api, state::AppState, validate::Validate};

#[derive(Default, Serialize)]
pub struct ValidJson<T>(pub T);

impl<T> FromRequest<AppState> for ValidJson<T>
where
    T: Validate + DeserializeOwned + Send + Sync + 'static,
{
    type Rejection = api::ErrorResponse;

    async fn from_request(
        req: axum::extract::Request,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Json(data) = <Json<T> as FromRequest<AppState>>::from_request(req, state).await?;

        let db_conn = state.db_conn().await?;

        db_conn
            .interact(move |db_conn| {
                data.validate(db_conn)?;
                Ok(Self(data))
            })
            .await?
    }
}
