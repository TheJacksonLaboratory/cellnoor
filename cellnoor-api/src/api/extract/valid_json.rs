use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Request},
};
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

#[derive(Default, Serialize)]
pub struct ValidPathJson<T, U>(pub T, pub U);

impl<T, U> FromRequest<AppState> for ValidPathJson<T, U>
where
    T: FromRequestParts<AppState> + Send + 'static,
    api::ErrorResponse: From<<T as FromRequestParts<AppState>>::Rejection>,
    U: DeserializeOwned + Send + Sync + 'static,
    (T, U): Validate,
{
    type Rejection = api::ErrorResponse;

    async fn from_request(
        req: axum::extract::Request,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let (mut request_parts, request_body) = req.into_parts();

        let path = T::from_request_parts(&mut request_parts, state).await?;
        let Json(data) = <Json<U> as FromRequest<AppState>>::from_request(
            Request::from_parts(request_parts, request_body),
            state,
        )
        .await?;

        let db_conn = state.db_conn().await?;

        db_conn
            .interact(move |db_conn| {
                let path_and_data = (path, data);
                path_and_data.validate(db_conn)?;
                let (path, data) = path_and_data;
                Ok(Self(path, data))
            })
            .await?
    }
}
