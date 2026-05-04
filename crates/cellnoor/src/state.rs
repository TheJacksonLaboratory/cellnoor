use std::sync::Arc;

use deadpool_postgres::PoolError;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::{
    db::{self},
    settings::Settings,
};

#[derive(Clone)]
pub struct DevState {
    db_pool: db::Pool,
}
impl DevState {
    pub async fn db_client(&self) -> Result<db::Client, PoolError> {
        self.db_pool.get(db::User::Person(Uuid::nil())).await
    }
}

#[derive(Clone)]
pub struct ProdState {
    db_pool: db::Pool,
    // Store these two things in one `Arc` instead of 2
    jwt_decoding_key: Arc<jsonwebtoken::DecodingKey>,
}

impl ProdState {
    pub async fn db_client(&self, user: db::User) -> Result<db::Client, PoolError> {
        self.db_pool.get(user).await
    }

    pub fn jwt_decoding_key(&self) -> &jsonwebtoken::DecodingKey {
        &self.jwt_decoding_key
    }
}

#[derive(Clone)]
pub enum AppState {
    Dev(DevState),
    Prod(ProdState),
}

impl AppState {
    pub fn initialize(settings: Settings) -> anyhow::Result<Self> {
        let db_pool = db::Pool::new(
            settings.db_url().expose_secret(),
            settings.max_db_pool_size(),
        )?;

        let state = if settings.with_auth() {
            Self::Prod(ProdState {
                db_pool,
                jwt_decoding_key: Arc::new(jsonwebtoken::DecodingKey::from_secret(
                    settings.auth_secret().expose_secret().as_bytes(),
                )),
            })
        } else {
            Self::Dev(DevState { db_pool })
        };

        Ok(state)
    }

    #[cfg(test)]
    fn initialize_for_test() -> Self {
        use std::env;

        use anyhow::Context;

        dotenvy::dotenv()?;

        let db_pool = db::Pool::new(
            &env::var("CELLNOOR_TEST_DB_URL")
                .context("environment variables 'CELLNOOR_TEST_DB_URL' required for test")?,
            None,
        )
        .unwrap();

        // Unit-tests don't use JSON web tokens, so we pass in an empty secret
        Self::Prod(ProdState {
            db_pool,
            jwt_decoding_key: Arc::new(jsonwebtoken::DecodingKey::from_secret(&[])),
        })
    }

    pub async fn db_client(
        &self,
        user: db::User,
    ) -> Result<db::Client, deadpool_postgres::PoolError> {
        match self {
            Self::Dev(s) => s.db_client().await,
            Self::Prod(s) => s.db_client(user).await,
        }
    }
}

#[cfg(test)]
pub static TEST_STATE: std::sync::LazyLock<AppState> =
    std::sync::LazyLock::new(AppState::initialize_for_test);
