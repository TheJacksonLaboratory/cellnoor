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

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[schemars(inline)]
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
    State(state): State<AppState>,
    user: AuthUser,
    Path(DatasetDir {
        dataset_type,
        dataset_id,
        _file_path,
    }): Path<DatasetDir>,
) -> Result<(), Error> {
    // If we know the user is staff without hitting the db, we can just return early
    if user.is_staff().is_some_and(|is| is) {
        return Ok(());
    }

    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    dataset_exists(tx, dataset_type, dataset_id)
        .await?
        .then_some(())
        .ok_or(ErrorInner::ResourceNotFound.into())
}

// Postgres row-level security will automatically hide what the user can't see (and it takes into account whether a user is staff)
async fn dataset_exists(
    tx: db::Transaction<'_>,
    dataset_type: DatasetType,
    dataset_id: Uuid,
) -> Result<bool, Error> {
    let exists = match dataset_type {
        DatasetType::ChromiumDatasets => chromium_dataset_exists(tx, dataset_id).await?,
    };

    Ok(exists)
}

async fn chromium_dataset_exists(
    tx: db::Transaction<'_>,
    dataset_id: Uuid,
) -> Result<bool, ErrorInner> {
    // We query chromium_dataset_to_specimen because that's accessible and has row-level security enabled
    static SELECT_DATASET: SqlBuilder = SqlBuilder::new(
        "select exists (select 1 from chromium_dataset_to_specimen where (chromium_dataset).id = $1)",
    );

    Ok(tx
        .query_one_into(&SELECT_DATASET.finish_with_params(vec![&dataset_id]))
        .await?)
}
