// The three fields not zeroized in `Config` cause a linting error
#![allow(unused)]
use std::{path::Path, str::FromStr};

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use zeroize::Zeroize;

use crate::initial_data::InitialData;

#[derive(Debug, Zeroize)]
pub struct Config {
    #[zeroize(skip)]
    mode: AppMode,
    db_root_user: String,
    db_root_password: String,
    cellnoor_api_db_password: String,
    cellnoor_ui_db_password: String,
    db_host: String,
    db_port: u16,
    db_name: String,
    api_key_prefix_length: usize,
    host: String,
    port: u16,
    #[zeroize(skip)]
    initial_data: InitialData,
    #[zeroize(skip)]
    log_dir: Option<Utf8PathBuf>,
}

impl Config {
    pub fn read() -> anyhow::Result<Self> {
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
            api_key_prefix_length,
            host,
            port,
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
            api_key_prefix_length: api_key_prefix_length
                .or_load(config_dir.join("api_key_prefix_length"))?,
            host: host.or_load(config_dir.join("host"))?,
            port: port.or_load(config_dir.join("port"))?,
            initial_data: None.or_load(config_dir.join("initial_data"))?,
            log_dir: log_dir.or_load(config_dir.join("log_dir")).ok(),
        })
    }

    #[must_use]
    pub fn cellnoor_api_db_password(&self) -> &str {
        &self.cellnoor_api_db_password
    }

    #[must_use]
    pub fn cellnoor_ui_db_password(&self) -> &str {
        &self.cellnoor_ui_db_password
    }

    fn db_url(&self, database_user: DatabaseUser) -> String {
        let Self {
            db_root_user,
            db_root_password,
            cellnoor_api_db_password,
            db_host,
            db_port,
            db_name,
            mode: _,
            cellnoor_ui_db_password: _,
            api_key_prefix_length: _,
            host: _,
            port: _,
            initial_data: _,
            log_dir: _,
        } = self;

        let base = "postgres://";
        let db_spec = format!("{db_host}:{db_port}/{db_name}");

        match database_user {
            DatabaseUser::Root => {
                format!("{base}{db_root_user}:{db_root_password}@{db_spec}")
            }
            DatabaseUser::CellnoorApi => {
                format!("{base}cellnoor_api:{cellnoor_api_db_password}@{db_spec}")
            }
        }
    }

    #[must_use]
    pub fn db_root_url(&self) -> String {
        self.db_url(DatabaseUser::Root)
    }

    #[must_use]
    pub fn cellnoor_api_db_url(&self) -> String {
        self.db_url(DatabaseUser::CellnoorApi)
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
    pub fn address(&self) -> String {
        let Self {
            host,
            port,
            mode: _,
            db_root_user: _,
            db_root_password: _,
            cellnoor_api_db_password: _,
            cellnoor_ui_db_password: _,
            db_host: _,
            db_port: _,
            db_name: _,
            api_key_prefix_length: _,
            initial_data: _,
            log_dir: _,
        } = self;

        format!("{host}:{port}")
    }

    #[must_use]
    pub fn api_key_prefix_length(&self) -> usize {
        self.api_key_prefix_length
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
    db_root_password: Option<String>,
    #[arg(long, env = "CELLNOOR_API_DB_PASSWORD")]
    cellnoor_api_db_password: Option<String>,
    #[arg(long, env = "CELLNOOR_UI_DB_PASSWORD")]
    cellnoor_ui_db_password: Option<String>,
    #[arg(long, env = "CELLNOOR_DB_HOST")]
    db_host: Option<String>,
    #[arg(long, env = "CELLNOOR_DB_PORT")]
    db_port: Option<u16>,
    #[arg(long, env = "CELLNOOR_DB_NAME")]
    db_name: Option<String>,
    #[arg(long, env = "CELLNOOR_API_KEY_PREFIX_LENGTH")]
    api_key_prefix_length: Option<usize>,
    #[arg(long, env = "CELLNOOR_API_HOST")]
    host: Option<String>,
    #[arg(long, env = "CELLNOOR_API_PORT")]
    port: Option<u16>,
    #[arg(long, env = "CELLNOOR_LOG_DIR")]
    log_dir: Option<Utf8PathBuf>,
}

trait OptionExt<T> {
    fn or_load<P>(self, path: P) -> anyhow::Result<T>
    where
        T: FromStr,
        T::Err: Send + Sync + std::error::Error + std::fmt::Display + 'static,
        P: std::fmt::Display + AsRef<Path>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_load<P>(self, path: P) -> anyhow::Result<T>
    where
        T: FromStr,
        T::Err: Send + Sync + std::error::Error + 'static,
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
