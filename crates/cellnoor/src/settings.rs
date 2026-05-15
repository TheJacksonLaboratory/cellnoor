use anyhow::Context;
use camino::Utf8Path;
use secrecy::SecretString;

fn default_with_auth() -> bool {
    true
}

fn default_address() -> String {
    "localhost:8000".to_owned()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    db_url: SecretString,
    auth_secret: SecretString,
    #[allow(dead_code)]
    auth_url: String,
    max_db_pool_size: Option<usize>,
    #[serde(default = "default_address")]
    address: String,
    #[allow(dead_code)]
    app_url: String,
    #[serde(default = "default_with_auth")]
    with_auth: bool,
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
                "failed to read app configuration from {config_path} and environment. All \
                 settings set in environment should have a prefix of 'CELLNOOR'"
            )
        } else {
            "failed to read app configuration from environment. All settings set in environment \
             should have a prefix of 'CELLNOOR'"
                .to_string()
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
    pub fn auth_secret(&self) -> &SecretString {
        &self.auth_secret
    }

    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    #[must_use]
    pub fn max_db_pool_size(&self) -> Option<usize> {
        self.max_db_pool_size
    }

    #[must_use]
    pub fn with_auth(&self) -> bool {
        self.with_auth
    }
}
