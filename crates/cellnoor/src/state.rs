use std::sync::Arc;

use secrecy::ExposeSecret;

use crate::{db, settings::Settings};

#[derive(Clone)]
pub struct DevState {
    db_pool: deadpool_postgres::Pool,
}

#[derive(Clone)]
pub struct ProdState {
    db_pool: deadpool_postgres::Pool,
    jwt_decoding_key: Arc<jsonwebtoken::DecodingKey>,
}

impl ProdState {
    fn jwt_decoding_key(&self) -> &jsonwebtoken::DecodingKey {
        &self.jwt_decoding_key
    }
}

#[derive(Clone)]
pub enum AppState {
    Dev(DevState),
    Prod(ProdState),
}

impl AppState {
    pub async fn initialize(settings: Settings) -> anyhow::Result<Self> {
        let db_pool = db::create_pool(
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

    pub async fn db_conn(&self) -> Result<deadpool_postgres::Object, deadpool_postgres::PoolError> {
        match self {
            Self::Dev(DevState { db_pool }) | Self::Prod(ProdState { db_pool, .. }) => {
                db_pool.get().await
            }
        }
    }
}
