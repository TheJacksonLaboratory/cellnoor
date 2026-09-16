use axum::{Json, extract::State};
use cellnoor_types::{
    cdna::{CdnaDetailed, CdnaQuery, SavedCdnaRecord},
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::{
        cdna::index_compact::cdna_simple_links,
        suspension_pools::index_compact::tagged_specimen_from_record,
    },
    state::AppState,
};

pub async fn index_cdna_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<CdnaQuery>,
) -> Result<Json<Vec<CdnaDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_cdna_detailed(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_cdna_detailed(
    tx: &db::Transaction<'_>,
    query: &CdnaQuery,
) -> Result<Vec<CdnaDetailed>, ErrorInner> {
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
        links: cdna_simple_links(*record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
}
