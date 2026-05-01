use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{RequestPartsExt, extract::FromRequestParts, http::HeaderValue, response::IntoResponse};
use axum_extra::TypedHeader;
use postgres_types::FromSql;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{self, Error},
    state::{AppState, ProdState},
};

const API_KEY_LENGTH: usize = 32;

#[derive(FromSql)]
#[postgres(name = "hashed_api_key_stub")]
pub struct HashedApiKey {
    id: Uuid,
    person_id: Option<Uuid>,
    hashed_key: String,
    expires_at: jiff::Timestamp,
}

impl HashedApiKey {
    fn is_person(&self) -> bool {
        self.person_id.is_some()
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < jiff::Timestamp::now()
    }

    pub fn to_user(&self) -> AuthUser {
        let Self { id, .. } = self;
        if self.is_person() {
            AuthUser::Person(*id)
        } else {
            AuthUser::Service(*id)
        }
    }
}

pub async fn fetch_user_id_by_api_key(
    api_key: &str,
    mut db_client: db::Client,
) -> Result<HashedApiKey, Error> {
    if api_key.len() != API_KEY_LENGTH {
        return Err(Error::invalid_api_key());
    }

    let prefix = &api_key[..8];

    let tx = db_client.begin().await?;

    let hashed_api_key = tx
        .query_one(
            "select (id, hashed_key, expires_at)::api_key_verification from api_key where prefix \
             = $1",
            &[&prefix],
        )
        .await
        .map(|r| r.get(0))?;

    Ok(hashed_api_key)
}

pub fn verify_api_key(
    verifier: &Argon2<'_>,
    api_key: &str,
    hashed_api_key: &HashedApiKey,
) -> Result<(), Error> {
    if hashed_api_key.is_expired() {
        return Err(Error::expired_api_key(hashed_api_key.expires_at));
    }

    let hashed_key = PasswordHash::new(&hashed_api_key.hashed_key)
        .map_err(|e| e.to_string())
        .map_err(Error::other)?;

    verifier
        .verify_password(api_key.as_bytes(), &hashed_key)
        .map_err(|_| Error::invalid_api_key())?;

    Ok(())
}
