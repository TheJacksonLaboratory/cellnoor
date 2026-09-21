use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    cdna::{CdnaDetailed, CdnaQuery, SavedCdnaRecord},
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    handlers::suspension_pools::index_compact::tagged_specimen_from_record,
    state::AppState,
};

pub async fn index_cdna_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<CdnaQuery>,
) -> Result<Json<Vec<CdnaDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| select_cdna_detailed(tx, &query).await)
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_cdna_detailed(
    tx: &db::Transaction<'_>,
    query: &CdnaQuery,
) -> Result<Vec<CdnaDetailed>, DbError> {
    static SELECT_DETAILED_CDNA: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select_rows(&SELECT_DETAILED_CDNA, query)
        .await?
        .into_iter()
        .map(map_detailed_row)
        .collect())
}

fn map_detailed_row(row: Row) -> CdnaDetailed {
    let record: SavedCdnaRecord = row.get("cdna");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");

    CdnaDetailed {
        links: SimpleLinks::from_str_and_id("/cdna", record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
}
