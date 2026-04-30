use anyhow::{Context, anyhow};

use crate::{
    db::{self, DbConnection, DbConnectionPool},
    initial_data::insert_initial_data,
    settings::{AppMode, Config},
};

#[derive(Clone)]
pub struct DevelopmentState {
    db_pool: DbConnectionPool,
    tokio_pg_pool: deadpool_postgres::Pool,
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
    tokio_pg_pool: deadpool_postgres::Pool,
    jwt_decoding_key: Arc<Mutex<Option<JwtDecodingKey>>>,
    jwt_validation: Arc<Validation>,
}

impl ProductionState {
    async fn new_jwk(&self) -> Result<JwtDecodingKey, db::Error> {
        let mut db_conn = self.db_pool.get().await?;

        let (expires_at, public_key): (jiff_diesel::Timestamp, String) = json_web_keys::table
            .select((json_web_keys::expires_at, json_web_keys::public_key))
            .filter(json_web_keys::expires_at.gt(jiff::Timestamp::now().to_diesel()))
            .order_by(json_web_keys::expires_at.desc())
            .first(&mut db_conn)
            .await?;

        Ok(JwtDecodingKey {
            expires_at: expires_at.to_jiff(),
            public_key: DecodingKey::from_jwk(&serde_json::from_str(&public_key).unwrap()).unwrap(),
        })
    }

    pub async fn jwt_decoding_key(
        &self,
    ) -> Result<MutexGuard<'_, Option<JwtDecodingKey>>, db::Error> {
        let Self {
            jwt_decoding_key, ..
        } = self;

        let mut maybe_jwk = jwt_decoding_key.lock().await;

        if maybe_jwk.as_ref().is_some_and(|key| !key.is_expired()) {
            return Ok(maybe_jwk);
        }

        let new_jwk = self.new_jwk().await?;

        *maybe_jwk = Some(new_jwk);

        Ok(maybe_jwk)
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

fn create_tokio_pg_pool(
    db_url: &SecretString,
    max_size: Option<usize>,
) -> anyhow::Result<deadpool_postgres::Pool> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = db_url.expose_secret().to_owned().into();

    let mut builder = cfg.builder(tokio_postgres::NoTls)?;
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

        let db_pool = create_db_pool(&db_url, config.max_db_pool_size())?;

        let state = match config.mode() {
            AppMode::Development => Self::Development(DevelopmentState {
                db_pool,
                tokio_pg_pool: create_tokio_pg_pool(&db_url, config.max_db_pool_size())?,
            }),
            AppMode::Production => {
                let mut jwt_validation = Validation::new(Algorithm::EdDSA);

                jwt_validation.set_audience(&[config.jwt_audience()]);
                jwt_validation.set_issuer(&[config.jwt_issuer()]);

                Self::Production(ProductionState {
                    db_pool,
                    tokio_pg_pool: create_tokio_pg_pool(&db_url, config.max_db_pool_size())?,
                    jwt_decoding_key: Arc::new(Mutex::new(None)),
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

    pub async fn tokio_pg_client(&self) -> Result<deadpool_postgres::Object, db::Error> {
        match self {
            Self::Development(DevelopmentState { tokio_pg_pool, .. })
            | Self::Production(ProductionState { tokio_pg_pool, .. }) => {
                tokio_pg_pool.get().await.map_err(db::Error::from)
            }
        }
    }
}
