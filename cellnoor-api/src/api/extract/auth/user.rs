use axum::extract::FromRequestParts;
use cellnoor_schema::api_keys;
use diesel::{PgConnection, prelude::*};
use uuid::Uuid;

use crate::{
    api::{
        self,
        extract::auth::{self, api_key::AsApiKey},
    },
    state::AppState,
};

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct AuthenticatedUser(Uuid);

impl AuthenticatedUser {
    pub fn id(&self) -> Uuid {
        self.0
    }

    fn fetch_by_api_key<T>(
        api_key: &T,
        prefix_length: usize,
        conn: &mut PgConnection,
    ) -> Result<Self, auth::Error>
    where
        T: AsApiKey,
    {
        let api_key_prefix = api_key.prefix(prefix_length);
        tracing::debug!(api_key_prefix);

        let (user_id, hash): (Uuid, String) = api_keys::table
            .filter(api_keys::prefix.eq(api_key_prefix))
            .select((api_keys::user_id, api_keys::hash))
            .first(conn)
            .optional()?
            .ok_or_else(auth::Error::invalid_api_key)?;

        tracing::debug!(user_id = user_id.to_string());

        if !api_key.is_same_hash(&hash) {
            return Err(auth::Error::invalid_api_key());
        }

        Ok(Self(user_id))
    }
}

const API_KEY_HEADER: &str = "X-API-Key";

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = api::ErrorResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState,
    ) -> Result<Self, api::ErrorResponse> {
        let api_key_prefix_length = match app_state {
            AppState::Production {
                api_key_prefix_length,
                db_pool: _,
            } => *api_key_prefix_length,
            AppState::Development {
                user_id,
                db_pool: _,
            } => {
                return Ok(Self(*user_id));
            }
        };

        let headers = &mut parts.headers;

        let Some(api_key) = headers.get(API_KEY_HEADER) else {
            return Err(auth::Error::no_api_key())?;
        };

        let mut decoded: [u8; 32] = [0; 32];
        base16ct::lower::decode(api_key, &mut decoded)
            .map_err(|_| auth::Error::invalid_api_key())?;

        let db_conn = app_state.db_conn().await?;

        Ok(db_conn
            .interact(move |db_conn| {
                AuthenticatedUser::fetch_by_api_key(&decoded, api_key_prefix_length, db_conn)
            })
            .await??)
    }
}
