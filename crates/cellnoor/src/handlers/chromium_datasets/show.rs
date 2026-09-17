use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetPredicateInner, ChromiumDatasetQuery,
    },
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, chromium_datasets::index_detailed::select_chromium_datasets_detailed},
    state::AppState,
};

pub async fn show_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<ChromiumDatasetDetailed>, Error> {
    state
        .in_transaction(user, async |tx| {
            select_chromium_dataset_by_id(tx, &state.public_files_url, id).await
        })
        .await
}

pub(super) async fn select_chromium_dataset_by_id(
    tx: &db::Transaction<'_>,
    raw_files_url: &str,
    id: Uuid,
) -> Result<ChromiumDatasetDetailed, ErrorInner> {
    let query = ChromiumDatasetQuery::from_filter(
        ChromiumDatasetPredicateInner::Id(UuidOperator::Eq(id)).into(),
    );

    let mut results = select_chromium_datasets_detailed(tx, raw_files_url, &query).await?;

    if results.len() != 1 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(results.swap_remove(0))
}
