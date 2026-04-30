use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use secrecy::SecretString;

fn true_() -> bool {
    true
}

fn localhost() -> String {
    "localhost".to_owned()
}

fn port_8000() -> u16 {
    8000
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    db_url: SecretString,
    auth_secret: SecretString,
    auth_url: String,
    max_db_pool_size: Option<usize>,
    #[serde(default = "localhost")]
    host: String,
    #[serde(default = "port_8000")]
    port: u16,
    app_url: String,
    #[serde(default = "true_")]
    with_auth: bool,
    log_dir: Option<Utf8PathBuf>,
}

impl Settings {
    pub fn read(config_path: Option<&Utf8Path>) -> anyhow::Result<Self> {
        use config::Config;

        let mut settings = Config::builder();

        if let Some(config_path) = config_path {
            settings = settings.add_source(config::File::new(
                config_path.as_str(),
                config::FileFormat::Toml,
            ));
        }

        settings = settings.add_source(config::Environment::with_prefix("CELLNOOR"));

        let err_message = if let Some(config_path) = config_path {
            format!(
                "failed to read app configuration from {config_path} and environment. All settings set in environment should have a prefix of 'CELLNOOR'"
            )
        } else {
            format!(
                "failed to read app configuration from environment. All settings set in environment should have a prefix of 'CELLNOOR'"
            )
        };

        let settings = settings
            .build()
            .context(err_message)
            .map(Config::try_deserialize)??;

        Ok(settings)
    }

    #[must_use]
    pub fn db_url(&self) -> &SecretString {
        &self.db_url
    }

    #[must_use]
    pub fn max_db_pool_size(&self) -> Option<usize> {
        self.max_db_pool_size
    }

    #[must_use]
    pub fn log_dir(&self) -> Option<&Utf8Path> {
        self.log_dir.as_ref().map(Utf8PathBuf::as_path)
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn with_auth(&self) -> bool {
        self.with_auth
    }
}
