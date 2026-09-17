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
    pub db: deadpool_postgres::Config,
    #[serde(default)]
    pub auth_secret: SecretString,
    pub max_db_pool_size: Option<usize>,
    #[serde(default = "default_address")]
    pub listen_on: String,
    pub public_files_url: String,
    pub public_auth_url: String,
    pub static_files_dir: String,
    #[serde(default = "default_with_auth")]
    pub with_auth: bool,
}

impl Settings {
    pub fn read() -> anyhow::Result<Self> {
        use config::{Config, Environment};

        let separator = "__";
        let mut settings: Settings = Config::builder()
            .add_source(Environment::with_prefix("CELLNOOR_APP").separator(separator))
            .add_source(Environment::with_prefix("CELLNOOR").separator(separator))
            .add_source(Environment::default().separator(separator))
            .build()
            .map(Config::try_deserialize)??;

        if settings.db.password.is_none() {
            settings.db.password = fs::read_to_string("/run/secrets/app_db_password").ok();
        }

        if settings.auth_secret.expose_secret().is_empty() {
            settings.auth_secret = fs::read_to_string("/run/secrets/auth_secret")
                .map(SecretString::from)
                .context("failed to read auth secret from environment and/or secret file")?;
        }

        settings.db.user.get_or_insert("app".to_owned());

        Ok(settings)
    }
}
