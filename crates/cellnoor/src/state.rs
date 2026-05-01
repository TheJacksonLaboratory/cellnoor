use std::sync::Arc;

use argon2::Argon2;
use deadpool_postgres::PoolError;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::{
    db::{self, User},
    settings::Settings,
};

#[derive(Clone)]
pub struct DevState {
    db_pool: db::Pool,
}
impl DevState {
    pub async fn db_client(&self) -> Result<db::Client, PoolError> {
        self.db_pool.get(User::Person(Uuid::nil())).await
    }
}

#[derive(Clone)]
pub struct ProdState {
    db_pool: db::Pool,
    // Store these two things in one `Arc` instead of 2
    auth: Arc<(jsonwebtoken::DecodingKey, Argon2<'static>)>,
}

impl ProdState {
    pub async fn db_client(&self, user: User) -> Result<db::Client, PoolError> {
        self.db_pool.get(user).await
    }

    pub fn jwt_decoding_key(&self) -> &jsonwebtoken::DecodingKey {
        &self.auth.0
    }

    pub fn api_key_verifier(&self) -> &Argon2 {
        &self.auth.1
    }
}

#[derive(Clone)]
pub enum AppState {
    Dev(DevState),
    Prod(ProdState),
}

impl AppState {
    pub async fn initialize(settings: Settings) -> anyhow::Result<Self> {
        let db_pool = db::Pool::new(
            settings.db_url().expose_secret(),
            settings.max_db_pool_size(),
        )?;

        let state = if settings.with_auth() {
            Self::Prod(ProdState {
                db_pool,
                auth: Arc::new((
                    jsonwebtoken::DecodingKey::from_secret(
                        settings.auth_secret().expose_secret().as_bytes(),
                    ),
                    Argon2::default(),
                )),
            })
        } else {
            Self::Dev(DevState { db_pool })
        };

        Ok(state)
    }

    pub async fn db_client(&self, user: User) -> Result<db::Client, deadpool_postgres::PoolError> {
        match self {
            Self::Dev(s) => s.db_client().await,
            Self::Prod(s) => s.db_client(user).await,
        }
    }
}
