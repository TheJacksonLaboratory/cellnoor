use axum::extract::{Path, State};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::file_auth::FileAuthError,
    state::AppState,
};

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[schemars(inline)]
#[strum(serialize_all = "snake_case")]
pub enum DatasetType {
    ChromiumDatasets,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[schemars(inline)]
pub struct DatasetDir {
    dataset_type: DatasetType,
    dataset_id: Uuid,
    _file_path: Option<String>,
}

#[axum::debug_handler]
pub async fn authorize_dataset_dir_access(
    state: State<AppState>,
    user: AuthUser,
    Path(DatasetDir {
        dataset_type,
        dataset_id,
        _file_path,
    }): Path<DatasetDir>,
) -> Result<(), FileAuthError> {
    tracing::debug!(
        %dataset_type,
        %dataset_id,
        file_path = _file_path.unwrap_or_default()
    );

    // If we know the user is staff, we can just return early
    if user.is_staff() {
        return Ok(());
    }

    let mut client = state.db_client(user).await.map_err(DbError::from)?;
    let tx = client.begin().await.map_err(DbError::from)?;

    dataset_exists(tx, dataset_type, dataset_id)
        .await?
        .then_some(())
        .ok_or(FileAuthError::DatasetAccessDenied)
}

// Postgres row-level security will automatically hide what the user can't see
// (and it takes into account whether a user is staff)
async fn dataset_exists(
    tx: db::Transaction<'_>,
    dataset_type: DatasetType,
    dataset_id: Uuid,
) -> Result<bool, DbError> {
    let exists = match dataset_type {
        DatasetType::ChromiumDatasets => chromium_dataset_exists(tx, dataset_id).await?,
    };

    Ok(exists)
}

async fn chromium_dataset_exists(
    tx: db::Transaction<'_>,
    dataset_id: Uuid,
) -> Result<bool, DbError> {
    // We query chromium_dataset_to_specimen because that's accessible and has
    // row-level security enabled
    static SELECT_DATASET: &str = "select exists (select 1 from chromium_dataset_to_specimen \
                                   where (chromium_dataset).id = $1)";

    tx.query_one_into(&Sql::new(SELECT_DATASET, vec![&dataset_id]))
        .await
}
