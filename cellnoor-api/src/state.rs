use std::sync::{Arc, RwLock, RwLockReadGuard};

use anyhow::{Context, anyhow};
use cellnoor_schema::json_web_keys;
use diesel::prelude::*;
use diesel_async::{
    AsyncConnection, AsyncMigrationHarness, AsyncPgConnection, RunQueryDsl,
    pooled_connection::AsyncDieselConnectionManager,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use jiff_diesel::ToDiesel;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    config::{AppMode, Config},
    db::{self, DbConnection, DbConnectionPool},
    initial_data::insert_initial_data,
};

#[derive(Clone)]
pub struct DevelopmentState {
    db_pool: DbConnectionPool,
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
    db_pool: DbConnectionPool,
    jwt_decoding_key: Arc<RwLock<Option<JwtDecodingKey>>>,
    jwt_validation: Arc<Validation>,
}

impl ProductionState {
    async fn new_jwk(&self) -> Result<JwtDecodingKey, db::Error> {
        let mut db_conn = self.db_pool.get().await?;

        let (expires_at, public_key): (jiff_diesel::Timestamp, String) = json_web_keys::table
            .select((json_web_keys::expires_at, json_web_keys::public_key))
            .filter(json_web_keys::expires_at.gt(jiff::Timestamp::now().to_diesel()))
            .first(&mut db_conn)
            .await?;

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
            let maybe_jwk = jwt_decoding_key
                .read()
                .expect("should be able to acquire read-lock on JWK");

            maybe_jwk.as_ref().is_none_or(JwtDecodingKey::is_expired)
        };

        if need_new_jwk {
            let new_jwk = self.new_jwk().await?;
            let mut writelock = jwt_decoding_key
                .write()
                .expect("should be able to acquire write-lock on JWK");

            *writelock = Some(new_jwk);
        }

        Ok(self
            .jwt_decoding_key
            .read()
            .expect("should be able to acquire read-lock on JWK"))
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
pub fn create_test_db_pool(db_url: &SecretString) -> anyhow::Result<DbConnectionPool> {
    create_db_pool(db_url, None)
}

fn create_db_pool(
    db_url: &SecretString,
    max_size: Option<usize>,
) -> anyhow::Result<DbConnectionPool> {
    let manager = AsyncDieselConnectionManager::new(db_url.expose_secret());
    let mut builder = DbConnectionPool::builder(manager);

    if let Some(max_size) = max_size {
        builder = builder.max_size(max_size);
    }

    Ok(builder.build()?)
}

fn run_migrations(db_conn: AsyncPgConnection) -> anyhow::Result<AsyncPgConnection> {
    const MIGRATIONS: EmbeddedMigrations =
        embed_migrations!("../crates/cellnoor-schema/migrations");

    let mut migration_harness = AsyncMigrationHarness::new(db_conn);
    migration_harness
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!(e))?;

    Ok(migration_harness.into_inner())
}

async fn set_db_user_password(
    username: &str,
    password: &SecretString,
    mut db_conn: &AsyncPgConnection,
) -> anyhow::Result<()> {
    diesel::sql_query(format!(
        r#"alter user "{username}" with password '{}'"#,
        password.expose_secret()
    ))
    .execute(&mut db_conn)
    .await?;

    Ok(())
}

impl AppState {
    pub async fn initialize(config: Config) -> anyhow::Result<Self> {
        let root_db_conn = AsyncPgConnection::establish(config.db_root_url().expose_secret())
            .await
            .context("failed to connect to db as root to run migrations")?;

        let root_db_conn = run_migrations(root_db_conn)?;
        tracing::info!("ran database migrations");

        tokio::try_join!(
            set_db_user_password(
                "cellnoor_api",
                config.cellnoor_api_db_password(),
                &root_db_conn
            ),
            set_db_user_password(
                "cellnoor_ui",
                config.cellnoor_ui_db_password(),
                &root_db_conn
            )
        )?;
        tracing::info!("set password for database users 'cellnoor-api' and 'cellnoor-ui'");

        let initial_data = config.initial_data();
        insert_initial_data(initial_data, reqwest::Client::new(), &root_db_conn)
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

                jwt_validation.set_audience(&[config.jwt_audience()]);
                jwt_validation.set_issuer(&[config.jwt_issuer()]);

                Self::Production(ProductionState {
                    db_pool,
                    jwt_decoding_key: Arc::new(RwLock::new(None)),
                    jwt_validation: Arc::new(jwt_validation),
                })
            }
        };

        Ok(state)
    }

    pub async fn db_conn(&self) -> Result<DbConnection, db::Error> {
        match self {
            Self::Development(DevelopmentState { db_pool, .. })
            | Self::Production(ProductionState { db_pool, .. }) => {
                Ok(db_pool.get().await.map(DbConnection::new)?)
            }
        }
    }
}
