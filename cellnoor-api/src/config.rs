// TODO: split the different aspects of the configuration into different places
use std::{path::Path, str::FromStr};

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use secrecy::{ExposeSecret, SecretString};

use crate::initial_data::InitialData;

#[derive(Debug)]
pub struct Config {
    mode: AppMode,
    db_root_user: String,
    db_root_password: SecretString,
    cellnoor_api_db_password: SecretString,
    cellnoor_ui_db_password: SecretString,
    db_host: String,
    db_port: u16,
    db_name: String,
    max_db_pool_size: Option<usize>,
    jwt_audience: String,
    jwt_issuer: String,
    address: String,
    initial_data: InitialData,
    log_dir: Option<Utf8PathBuf>,
}

impl Config {
    pub fn read() -> anyhow::Result<Self> {
        dotenvy::dotenv().unwrap_or_default();

        let Cli {
            config_dir,
            mode,
            db_root_user,
            db_root_password,
            cellnoor_api_db_password,
            cellnoor_ui_db_password,
            db_host,
            db_port,
            db_name,
            max_db_pool_size,
            jwt_audience,
            jwt_issuer,
            address,
            log_dir,
        } = Cli::parse();

        Ok(Self {
            mode: mode.or_load(config_dir.join("mode")).unwrap_or_default(),
            db_root_user: db_root_user.or_load(config_dir.join("db_root_user"))?,
            db_root_password: db_root_password.or_load(config_dir.join("db_root_password"))?,
            cellnoor_api_db_password: cellnoor_api_db_password
                .or_load(config_dir.join("cellnoor_api_db_password"))?,
            cellnoor_ui_db_password: cellnoor_ui_db_password
                .or_load(config_dir.join("cellnoor_ui_db_password"))?,
            db_host: db_host.or_load(config_dir.join("db_host"))?,
            db_port: db_port.or_load(config_dir.join("db_port"))?,
            db_name: db_name.or_load(config_dir.join("db_name"))?,
            max_db_pool_size,
            jwt_audience: jwt_audience.or_load(config_dir.join("jwt_audience"))?,
            jwt_issuer: jwt_issuer.or_load(config_dir.join("jwt_issuer"))?,
            address: address.or_load(config_dir.join("address"))?,
            initial_data: None::<InitialData>.or_load(config_dir.join("initial_data"))?,
            log_dir: log_dir.or_load(config_dir.join("log_dir")).ok(),
        })
    }

    #[must_use]
    pub fn cellnoor_api_db_password(&self) -> &SecretString {
        &self.cellnoor_api_db_password
    }

    #[must_use]
    pub fn cellnoor_ui_db_password(&self) -> &SecretString {
        &self.cellnoor_ui_db_password
    }

    fn db_url(&self, database_user: DatabaseUser) -> SecretString {
        let Self {
            db_root_user,
            db_root_password,
            cellnoor_api_db_password,
            db_host,
            db_port,
            db_name,
            ..
        } = self;

        let base = "postgres://";
        let db_spec = format!("/{db_name}?host={db_host}&port={db_port}");

        match database_user {
            DatabaseUser::Root => format!(
                "{base}{db_root_user}:{}@{db_spec}",
                db_root_password.expose_secret()
            )
            .into(),
            DatabaseUser::CellnoorApi => format!(
                "{base}cellnoor_api:{}@{db_spec}",
                cellnoor_api_db_password.expose_secret()
            )
            .into(),
        }
    }

    #[must_use]
    pub fn db_host(&self) -> &str {
        &self.db_host
    }

    #[must_use]
    pub fn db_port(&self) -> u16 {
        self.db_port
    }

    #[must_use]
    pub fn db_root_user(&self) -> &str {
        &self.db_root_user
    }

    #[must_use]
    pub fn db_root_password(&self) -> &SecretString {
        &self.db_root_password
    }

    #[must_use]
    pub fn db_root_url(&self) -> SecretString {
        self.db_url(DatabaseUser::Root)
    }

    #[must_use]
    pub fn cellnoor_api_db_url(&self) -> SecretString {
        self.db_url(DatabaseUser::CellnoorApi)
    }

    #[must_use]
    pub fn max_db_pool_size(&self) -> Option<usize> {
        self.max_db_pool_size
    }

    #[must_use]
    pub fn jwt_audience(&self) -> &str {
        &self.jwt_audience
    }

    #[must_use]
    pub fn jwt_issuer(&self) -> &str {
        &self.jwt_issuer
    }

    #[must_use]
    pub fn initial_data(&self) -> InitialData {
        self.initial_data.clone()
    }

    #[must_use]
    pub fn log_dir(&self) -> Option<&Utf8Path> {
        self.log_dir.as_ref().map(Utf8PathBuf::as_path)
    }

    #[must_use]
    pub fn mode(&self) -> AppMode {
        self.mode
    }

    #[must_use]
    pub fn address(&self) -> &str {
        let Self { address, .. } = self;

        address
    }
}

#[derive(Clone, Copy)]
enum DatabaseUser {
    Root,
    CellnoorApi,
}

#[derive(Clone, Debug, Parser)]
struct Cli {
    #[arg(long, env = "CELLNOOR_CONFIG_DIR")]
    config_dir: Utf8PathBuf,
    #[arg(long, env = "CELLNOOR_MODE")]
    mode: Option<AppMode>,
    #[arg(long, env = "CELLNOOR_DB_ROOT_USER")]
    db_root_user: Option<String>,
    #[arg(long, env = "CELLNOOR_DB_ROOT_PASSWORD")]
    db_root_password: Option<SecretString>,
    #[arg(long, env = "CELLNOOR_API_DB_PASSWORD")]
    cellnoor_api_db_password: Option<SecretString>,
    #[arg(long, env = "CELLNOOR_UI_DB_PASSWORD")]
    cellnoor_ui_db_password: Option<SecretString>,
    #[arg(long, env = "CELLNOOR_DB_HOST")]
    db_host: Option<String>,
    #[arg(long, env = "CELLNOOR_DB_PORT")]
    db_port: Option<u16>,
    #[arg(long, env = "CELLNOOR_DB_NAME")]
    db_name: Option<String>,
    #[arg(long, env = "CELLNOOR_API_MAX_DB_POOL_SIZE")]
    max_db_pool_size: Option<usize>,
    #[arg(long, env = "CELLNOOR_JWT_ISSUER")]
    jwt_issuer: Option<String>,
    #[arg(long, env = "CELLNOOR_JWT_AUDIENCE")]
    jwt_audience: Option<String>,
    #[arg(long, env = "CELLNOOR_API_ADDRESS")]
    address: Option<String>,
    #[arg(long, env = "CELLNOOR_LOG_DIR")]
    log_dir: Option<Utf8PathBuf>,
}

trait OptionExt<T> {
    fn or_load<P>(self, path: P) -> anyhow::Result<T>
    where
        P: std::fmt::Display + AsRef<Path>;
}

impl<T> OptionExt<T> for Option<T>
where
    T: FromStr,
    T::Err: Send + Sync + std::error::Error + 'static,
{
    fn or_load<P>(self, path: P) -> anyhow::Result<T>
    where
        P: std::fmt::Display + AsRef<Path>,
    {
        if let Some(value) = self {
            return Ok(value);
        }

        let contents = std::fs::read_to_string(&path)
            .context(format!("failed to read contents of file {path}"))?;

        contents.parse().context(format!(
            "failed to parse contents of {path} as {}",
            std::any::type_name::<T>()
        ))
    }
}

trait SecretStringExt {
    fn or_load<P>(self, path: P) -> anyhow::Result<SecretString>
    where
        P: std::fmt::Display + AsRef<Path>;
}

impl SecretStringExt for Option<SecretString> {
    fn or_load<P>(self, path: P) -> anyhow::Result<SecretString>
    where
        P: std::fmt::Display + AsRef<Path>,
    {
        if let Some(value) = self {
            return Ok(value);
        }

        std::fs::read_to_string(&path)
            .context(format!("failed to read contents of file {path}"))
            .map(SecretString::from)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AppMode {
    Development,
    #[default]
    Production,
}

#[derive(Debug, thiserror::Error)]
#[error("{0} is an invalid AppMode")]
pub struct ParseModeError(String);

impl FromStr for AppMode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            _ => Err(ParseModeError(s.to_owned())),
        }
    }
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => "development".fmt(f),
            Self::Production => "production".fmt(f),
        }
    }
}
