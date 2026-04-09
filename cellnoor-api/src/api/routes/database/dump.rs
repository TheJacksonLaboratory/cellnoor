use std::{io::Write, process::Command};

use axum::Json;
use axum_extra::TypedHeader;
use headers::ContentEncoding;
use secrecy::ExposeSecret;

use crate::config::Config;

#[derive(serde::Deserialize)]
pub(super) struct DbBackupQuery {
    #[serde(default)]
    data_only: bool,
}

fn map_err(e: impl std::error::Error) -> serde_json::Value {
    serde_json::json!({"error": {"message": "failed to dump database", "reason": e.to_string()}})
}

#[axum::debug_handler]
pub(super) async fn dump_database(
    axum::extract::Query(DbBackupQuery { data_only }): axum::extract::Query<DbBackupQuery>,
) -> Result<(TypedHeader<ContentEncoding>, Vec<u8>), Json<serde_json::Value>> {
    // Rather than pass around the configuration in `AppState`, we can just read it
    // when we need to. Also, `unwrap` is fine because at this point, we've already
    // checked that the configuration was parsed correctly
    let config = Config::read().unwrap();

    let mut cmd = Command::new("pg_dumpall");
    cmd.env("PGPASSWORD", config.db_root_password().expose_secret());
    cmd.args([
        "--dbname",
        config.db_root_url().expose_secret(),
        "--no-password",
    ]);

    if data_only {
        cmd.arg("--data-only");
    }

    let std::process::Output {
        status,
        stdout,
        stderr,
    } = cmd.output().map_err(map_err)?;

    if !status.success() {
        return Err(Json(
            serde_json::json!({"error": {"message": "failed to dump database", "reason": String::from_utf8(stderr).unwrap()}}),
        ));
    }

    let output_buffer = Vec::with_capacity(stdout.len());
    let mut encoder = zstd::Encoder::new(output_buffer, 22).unwrap();
    encoder.write_all(&stdout).map_err(map_err)?;

    let output = encoder.finish().map_err(map_err)?;

    Ok((TypedHeader(ContentEncoding::zstd()), output))
}
