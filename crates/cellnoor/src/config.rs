// use std::{path::Path, str::FromStr};

// use anyhow::Context;
// use camino::{Utf8Path, Utf8PathBuf};
// use clap::Parser;
// use secrecy::{ExposeSecret, SecretString};

// #[derive(Debug)]
// pub struct Config {
//     mode: AppMode,
//     db_url: SecretString,
//     auth_secret: SecretString,
//     auth_url: String,
//     max_db_pool_size: Option<usize>,
//     host: String,
//     port: u16,
//     log_dir: Option<Utf8PathBuf>,
// }

// impl Config {
//     pub fn read() -> anyhow::Result<Self> {
//         dotenvy::dotenv().unwrap_or_default();

//         let Cli {
//             config_dir,
//             db_url,
//             max_db_pool_size,
//             auth_secret,
//             host,
//             port,
//             log_dir,
//             with_authentication,
//         } = Cli::parse();

//         Ok(Self {
//             mode: mode.or_load(config_dir.join("mode")).unwrap_or_default(),
//             db_root_user: db_root_user.or_load(config_dir.join("db_root_user"))?,
//             db_root_password: db_root_password.or_load(config_dir.join("db_root_password"))?,
//             cellnoor_api_db_password: cellnoor_api_db_password
//                 .or_load(config_dir.join("cellnoor_api_db_password"))?,
//             cellnoor_ui_db_password: cellnoor_ui_db_password
//                 .or_load(config_dir.join("cellnoor_ui_db_password"))?,
//             db_host: db_host.or_load(config_dir.join("db_host"))?,
//             db_port: db_port.or_load(config_dir.join("db_port"))?,
//             db_name: db_name.or_load(config_dir.join("db_name"))?,
//             max_db_pool_size,
//             jwt_audience: jwt_audience.or_load(config_dir.join("jwt_audience"))?,
//             jwt_issuer: jwt_issuer.or_load(config_dir.join("jwt_issuer"))?,
//             address: address.or_load(config_dir.join("address"))?,
//             initial_data: None::<InitialData>.or_load(config_dir.join("initial_data"))?,
//             log_dir: log_dir.or_load(config_dir.join("log_dir")).ok(),
//         })
//     }

//     #[must_use]
//     pub fn db_url(&self) -> &SecretString {
//         todo!()
//     }

//     #[must_use]
//     pub fn max_db_pool_size(&self) -> Option<usize> {
//         self.max_db_pool_size
//     }

//     #[must_use]
//     pub fn log_dir(&self) -> Option<&Utf8Path> {
//         self.log_dir.as_ref().map(Utf8PathBuf::as_path)
//     }

//     #[must_use]
//     pub fn address(&self) -> &str {
//         let Self { address, .. } = self;

//         address
//     }
// }

// #[derive(Clone, Copy)]
// enum DatabaseUser {
//     Root,
//     CellnoorApi,
// }

// #[derive(Clone, Debug, Parser)]
// struct Cli {
//     #[arg(long, env = "CELLNOOR_APP_CONFIG_DIR")]
//     config_dir: Utf8PathBuf,
//     #[arg(long, env = "CELLNOOR_APP_DB_URL")]
//     db_url: Option<String>,
//     #[arg(long, env = "CELLNOOR_APP_MAX_DB_POOL_SIZE")]
//     max_db_pool_size: Option<usize>,
//     #[arg(long, env = "CELLNOOR_AUTH_SECRET")]
//     auth_secret: Option<SecretString>,
//     #[arg(long, env = "CELLNOOR_APP_HOST")]
//     host: Option<String>,
//     #[arg(long, env = "CELLNOOR_APP_PORT")]
//     port: Option<u16>,
//     #[arg(long, env = "CELLNOOR_LOG_DIR")]
//     log_dir: Option<Utf8PathBuf>,
//     #[arg(long, env = "CELLNOOR_APP_WITH_AUTHENTICATION", default_value_t = true)]
//     with_authentication: bool,
// }

// trait OptionExt<T> {
//     fn or_load<P>(self, path: P) -> anyhow::Result<T>
//     where
//         P: std::fmt::Display + AsRef<Path>;
// }

// impl<T> OptionExt<T> for Option<T>
// where
//     T: FromStr,
//     T::Err: Send + Sync + std::error::Error + 'static,
// {
//     fn or_load<P>(self, path: P) -> anyhow::Result<T>
//     where
//         P: std::fmt::Display + AsRef<Path>,
//     {
//         if let Some(value) = self {
//             return Ok(value);
//         }

//         let contents = std::fs::read_to_string(&path)
//             .context(format!("failed to read contents of file {path}"))?;

//         contents.parse().context(format!(
//             "failed to parse contents of {path} as {}",
//             std::any::type_name::<T>()
//         ))
//     }
// }

// trait SecretStringExt {
//     fn or_load<P>(self, path: P) -> anyhow::Result<SecretString>
//     where
//         P: std::fmt::Display + AsRef<Path>;
// }

// impl SecretStringExt for Option<SecretString> {
//     fn or_load<P>(self, path: P) -> anyhow::Result<SecretString>
//     where
//         P: std::fmt::Display + AsRef<Path>,
//     {
//         if let Some(value) = self {
//             return Ok(value);
//         }

//         std::fs::read_to_string(&path)
//             .context(format!("failed to read contents of file {path}"))
//             .map(SecretString::from)
//     }
// }
