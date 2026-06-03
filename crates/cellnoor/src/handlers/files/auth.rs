use std::convert::identity;

use axum::extract::{Path, State};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, SqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct FilePathParam {
    dataset_id: Uuid,
    path: Vec<String>,
}

#[axum::debug_handler]
pub async fn authenticate_file_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(FilePathParam { dataset_id, path }): Path<FilePathParam>,
) -> Result<(), Error> {
    // Staff can see any files
    if user_is_staff(&state, user).await? {
        return Ok(());
    }

    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    // The database has row-level security enabled, so it should suffice to just find whether the file exists or not
    if !file_exists(tx, dataset_id, &path.join("/")).await? {
        // We don't really care to return a proper permission denied error lol
        return Err(ErrorInner::ResourceNotFound.into());
    }

    Ok(())
}

// Note that we take the app state instead of a db client because we might not need to hit the db
async fn user_is_staff(state: &AppState, user: AuthUser) -> Result<bool, ErrorInner> {
    static SELECT_IS_STAFF: SqlBuilder = SqlBuilder::new("select current_user_is_staff()");

    if let Some(is_staff) = user.is_staff() {
        return Ok(is_staff);
    }

    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    Ok(tx
        .query_one_into(&SELECT_IS_STAFF.finish_with_params(vec![]))
        .await?)
}

async fn file_exists(
    tx: db::Transaction<'_>,
    dataset_id: Uuid,
    path: &str,
) -> Result<bool, ErrorInner> {
    // We have to check all of our experimental modalities here, so we do that asynchronously
    let checks = [chromium_dataset_file_exists(tx, dataset_id, path)];

    Ok(futures::future::try_join_all(checks)
        .await?
        .into_iter()
        .any(identity))
}

async fn chromium_dataset_file_exists(
    tx: db::Transaction<'_>,
    dataset_id: Uuid,
    path: &str,
) -> Result<bool, ErrorInner> {
    static SELECT_CHROMIUM_DATASET_RAW_FILE: SqlBuilder =
        SqlBuilder::new(include_str!("select_chromium_dataset_raw_file.sql"));

    Ok(tx
        .query_one_into(
            &SELECT_CHROMIUM_DATASET_RAW_FILE.finish_with_params(vec![&dataset_id, &path]),
        )
        .await?)
}
