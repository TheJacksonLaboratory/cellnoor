use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    chromium_dataset::{ChromiumDatasetDetailed, ChromiumDatasetPredicateInner},
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{
        IdParam, chromium_datasets::index_detailed::select_chromium_datasets_detailed,
    },
    state::AppState,
};

pub async fn show_chromium_dataset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<ChromiumDatasetDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_dataset_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(super) async fn select_chromium_dataset_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<ChromiumDatasetDetailed, ErrorInner> {
    select_one(
        tx,
        ChromiumDatasetPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_chromium_datasets_detailed,
    )
    .await
}
