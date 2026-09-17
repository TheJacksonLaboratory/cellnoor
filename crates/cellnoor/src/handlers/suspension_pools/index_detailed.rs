use axum::{Json, extract::State};
use cellnoor_types::suspension_pool::{
    SavedSuspensionPoolRecord, SavedTaggedSpecimenRecord, SuspensionPoolDetailed,
    SuspensionPoolQuery,
};
use deadpool_postgres::tokio_postgres::Row;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    handlers::suspension_pools::index_compact::{
        suspension_pool_links, tagged_specimen_from_record,
    },
    state::AppState,
};

pub async fn index_suspension_pools_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SuspensionPoolQuery>,
) -> Result<Json<Vec<SuspensionPoolDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| {
            select_suspension_pools_detailed(tx, &query).await
        })
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_suspension_pools_detailed(
    tx: &db::Transaction<'_>,
    query: &SuspensionPoolQuery,
) -> Result<Vec<SuspensionPoolDetailed>, DbError> {
    static SELECT_DETAILED_SUSPENSION_POOL: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select_rows(&SELECT_DETAILED_SUSPENSION_POOL, query)
        .await?
        .into_iter()
        .map(map_detailed_row)
        .collect())
}

fn map_detailed_row(row: Row) -> SuspensionPoolDetailed {
    let record: SavedSuspensionPoolRecord = row.get("suspension_pool");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");

    SuspensionPoolDetailed {
        links: suspension_pool_links(record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
}
