use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::chromium_dataset::{ChromiumDatasetDetailed, ChromiumDatasetUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{IdParam, chromium_datasets::show::select_chromium_dataset_by_id},
    state::AppState,
};

pub async fn update_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<ChromiumDatasetUpdate>,
) -> Result<Json<ChromiumDatasetDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_chromium_dataset_by_id(&tx, state.raw_files_url(), id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_chromium_dataset_by_id(
    tx: &db::Transaction<'_>,
    raw_files_url: &str,
    id: Uuid,
    update: &ChromiumDatasetUpdate,
) -> Result<ChromiumDatasetDetailed, ErrorInner> {
    db::update(tx, "chromium_dataset", id, update).await?;
    select_chromium_dataset_by_id(tx, raw_files_url, id).await
}
