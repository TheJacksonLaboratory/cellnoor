use std::fs;

use anyhow::Context;
use secrecy::{ExposeSecret, SecretString};

fn default_with_auth() -> bool {
    true
}

fn default_address() -> String {
    "localhost:8000".to_owned()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    db: deadpool_postgres::Config,
    #[serde(default)]
    auth_secret: SecretString,
    max_db_pool_size: Option<usize>,
    #[serde(default = "default_address")]
    listen_on: String,
    public_fiels_url: String,
    #[serde(default = "default_with_auth")]
    with_auth: bool,
}

impl Settings {
    pub fn read() -> anyhow::Result<Self> {
        use config::Config;

        let mut settings = Config::builder();

        settings = settings
            .add_source(config::Environment::with_prefix("CELLNOOR").separator("_"))
            .add_source(config::Environment::with_prefix("").separator("__"));

        let mut settings: Settings = settings
            .build().context("failed to read app configuration from environment. All settings set in environment must have a prefix of 'CELLNOOR'")
            .map(Config::try_deserialize)??;

        if settings.db.password.is_none() {
            settings.db.password = fs::read_to_string("/run/secrets/cellnoor_db_url").ok();
        }

        if settings.auth_secret.expose_secret().is_empty() {
            settings.auth_secret = fs::read_to_string("/run/secrets/cellnoor_auth_secret")
                .map(SecretString::from)
                .context("failed to read auth secret from environment and/or secret file")?;
        }

        settings.db.user.get_or_insert("app".to_owned());

        Ok(settings)
    }

    #[must_use]
    pub fn db_config(&self) -> &deadpool_postgres::Config {
        &self.db
    }

    #[must_use]
    pub fn auth_secret(&self) -> &SecretString {
        &self.auth_secret
    }

    #[must_use]
    pub fn listen_on(&self) -> &str {
        &self.listen_on
    }

    #[must_use]
    pub fn max_db_pool_size(&self) -> Option<usize> {
        self.max_db_pool_size
    }

    #[must_use]
    pub fn with_auth(&self) -> bool {
        self.with_auth
    }

    #[must_use]
    pub fn public_files_url(&self) -> &str {
        &self.public_fiels_url
    }
}
