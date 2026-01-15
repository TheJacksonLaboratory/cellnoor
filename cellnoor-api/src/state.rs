use std::sync::Arc;

use anyhow::{Context, anyhow};
use deadpool_diesel::{
    Runtime,
    postgres::{Manager as PoolManager, Pool},
};
use diesel::{PgConnection, prelude::*};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use jsonwebtoken::DecodingKey;
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use crate::{
    config::{AppMode, Config},
    db,
    initial_data::insert_initial_data,
};

#[derive(Clone)]
pub struct DevelopmentState {
    db_pool: Pool,
    user_id: Uuid,
}

impl DevelopmentState {
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
}

#[derive(Clone)]
pub struct ProductionState {
    db_pool: Pool,
    jwt_decoding_key: Arc<DecodingKey>,
}

impl ProductionState {
    pub fn jwt_decoding_key(&self) -> &DecodingKey {
        &self.jwt_decoding_key
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

fn create_dev_superuser(db_conn: &mut PgConnection) -> anyhow::Result<Uuid> {
    let user_id = Uuid::now_v7();

    diesel::sql_query(format!(r#"create user "{user_id}" with superuser"#))
        .execute(db_conn)
        .context("failed to create dev superuser")?;

    Ok(user_id)
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
            AppMode::Development => {
                let mut db_conn = PgConnection::establish(config.db_root_url().expose_secret())?;
                let user_id = create_dev_superuser(&mut db_conn)?;
                Self::Development(DevelopmentState { db_pool, user_id })
            }
            AppMode::Production => Self::Production(ProductionState {
                db_pool,
                jwt_decoding_key: Arc::new(DecodingKey::from_secret(
                    config.auth_secret().expose_secret().as_bytes(),
                )),
            }),
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
