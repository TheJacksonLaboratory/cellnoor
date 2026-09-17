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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_str_eq;

    use crate::settings::Settings;

    // `Settings::read` reads the process environment, so this is the only test
    // allowed to set these variables: a second one would race with it
    #[test]
    fn read_applies_defaults() {
        unsafe {
            std::env::set_var("CELLNOOR__DB__HOST", "localhost");
            std::env::set_var("CELLNOOR__AUTH_SECRET", "secret");
            std::env::set_var("CELLNOOR__PUBLIC_FILES_URL", "http://files.localhost");
            std::env::set_var("CELLNOOR__PUBLIC_AUTH_URL", "http://auth.localhost");
            std::env::set_var("CELLNOOR__STATIC_FILES_DIR", "static");
        }

        let settings = Settings::read().unwrap();

        assert_str_eq!(settings.listen_on, "localhost:8000");
        assert!(settings.with_auth);
        assert_str_eq!(settings.db.user.unwrap(), "app");
        assert_str_eq!(settings.db.host.unwrap(), "localhost");
    }
}
