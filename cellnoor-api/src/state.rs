use std::sync::Arc;

use anyhow::{Context, anyhow};
use cellnoor_schema::json_web_keys;
use deadpool_diesel::{
    Runtime,
    postgres::{Manager as PoolManager, Pool},
};
use diesel::{PgConnection, prelude::*};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use jiff_diesel::ToDiesel;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::{
    config::{AppMode, Config},
    db,
    initial_data::insert_initial_data,
};

#[derive(Clone)]
pub struct DevelopmentState {
    db_pool: Pool,
}

pub struct JwtDecodingKey {
    expires_at: jiff::Timestamp,
    public_key: DecodingKey,
}
impl JwtDecodingKey {
    fn is_expired(&self) -> bool {
        self.expires_at < jiff::Timestamp::now()
    }

    pub fn public_key(&self) -> &DecodingKey {
        &self.public_key
    }
}

#[derive(Clone)]
pub struct ProductionState {
    db_pool: Pool,
    jwt_decoding_key: Arc<RwLock<Option<JwtDecodingKey>>>,
    jwt_validation: Arc<Validation>,
}

impl ProductionState {
    async fn new_jwk(&self) -> Result<JwtDecodingKey, db::Error> {
        let db_conn = self.db_pool.get().await?;

        let (expires_at, public_key): (jiff_diesel::Timestamp, String) = db_conn
            .interact(|db_conn| {
                json_web_keys::table
                    .select((json_web_keys::expires_at, json_web_keys::public_key))
                    .filter(json_web_keys::expires_at.gt(jiff::Timestamp::now().to_diesel()))
                    .first(db_conn)
            })
            .await??;

        Ok(JwtDecodingKey {
            expires_at: expires_at.to_jiff(),
            public_key: DecodingKey::from_jwk(&serde_json::from_str(&public_key).unwrap()).unwrap(),
        })
    }

    pub async fn jwt_decoding_key(
        &self,
    ) -> Result<RwLockReadGuard<'_, Option<JwtDecodingKey>>, db::Error> {
        let Self {
            jwt_decoding_key, ..
        } = self;

        let need_new_jwk = {
            let maybe_jwk = jwt_decoding_key.read().await;

            maybe_jwk.as_ref().is_none_or(JwtDecodingKey::is_expired)
        };

        if need_new_jwk {
            let mut writelock = jwt_decoding_key.write().await;
            *writelock = Some(self.new_jwk().await?);
        }

        Ok(self.jwt_decoding_key.read().await)
    }

    pub fn jwt_validation(&self) -> &Validation {
        &self.jwt_validation
    }
}

#[derive(Clone)]
pub enum AppState {
    Development(DevelopmentState),
    Production(ProductionState),
}

#[cfg(any(feature = "dummy-data", test))]
pub fn create_test_db_pool(db_url: &SecretString) -> anyhow::Result<Pool> {
    create_db_pool(db_url, None)
}

fn create_db_pool(db_url: &SecretString, max_size: Option<usize>) -> anyhow::Result<Pool> {
    let manager = PoolManager::new(db_url.expose_secret(), Runtime::Tokio1);
    let mut builder = Pool::builder(manager);

    if let Some(max_size) = max_size {
        builder = builder.max_size(max_size);
    }

    Ok(builder.build()?)
}

fn run_migrations(db_conn: &mut PgConnection) -> anyhow::Result<()> {
    const MIGRATIONS: EmbeddedMigrations =
        embed_migrations!("../crates/cellnoor-schema/migrations");

    db_conn
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

fn set_db_user_password(
    username: &str,
    password: &SecretString,
    db_conn: &mut PgConnection,
) -> anyhow::Result<()> {
    diesel::sql_query(format!(
        r#"alter user "{username}" with password '{}'"#,
        password.expose_secret()
    ))
    .execute(db_conn)?;

    Ok(())
}

impl AppState {
    pub async fn initialize(config: Config) -> anyhow::Result<Self> {
        let mut root_db_conn = PgConnection::establish(config.db_root_url().expose_secret())
            .context("failed to connect to db as root to run migrations")?;

        run_migrations(&mut root_db_conn)?;
        tracing::info!("ran database migrations");

        let db_users = [
            ("cellnoor_api", config.cellnoor_api_db_password()),
            ("cellnoor_ui", config.cellnoor_ui_db_password()),
        ];
        for (username, password) in db_users {
            set_db_user_password(username, password, &mut root_db_conn)?;
            tracing::info!("set password for database user '{username}'");
        }

        // Get a connection pool as the root user so as to insert the initial data. We
        // only need one connection here
        let root_db_pool = create_db_pool(&config.db_root_url(), Some(1))?;
        let initial_data = config.initial_data();
        insert_initial_data(initial_data, reqwest::Client::new(), root_db_pool.clone())
            .await
            .context("failed to insert initial data")?;
        tracing::info!("inserted initial data");

        let db_url = match config.mode() {
            AppMode::Development => config.db_root_url(),
            AppMode::Production => config.cellnoor_api_db_url(),
        };

        let db_pool = create_db_pool(&db_url, None)?;

        let state = match config.mode() {
            AppMode::Development => Self::Development(DevelopmentState { db_pool }),
            AppMode::Production => {
                let mut jwt_validation = Validation::new(Algorithm::EdDSA);

                if let Some(api_url) = config.public_api_url() {
                    jwt_validation.set_audience(&[api_url]);
                }

                if let Some(ui_url) = config.public_ui_url() {
                    jwt_validation.set_issuer(&[ui_url]);
                }

                Self::Production(ProductionState {
                    db_pool,
                    jwt_decoding_key: Arc::new(RwLock::new(None)),
                    jwt_validation: Arc::new(jwt_validation),
                })
            }
        };

        Ok(state)
    }

    pub async fn db_conn(&self) -> Result<deadpool_diesel::postgres::Connection, db::Error> {
        match self {
            Self::Development(DevelopmentState { db_pool, .. })
            | Self::Production(ProductionState { db_pool, .. }) => Ok(db_pool.get().await?),
        }
    }
}
