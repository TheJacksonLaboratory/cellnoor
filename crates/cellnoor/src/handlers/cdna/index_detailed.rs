use axum::{Json, extract::State};
use cellnoor_types::{
    cdna::{CdnaDetailed, CdnaQuery, SavedCdnaRecord},
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

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
    Json(mut query): Json<CdnaQuery>,
) -> Result<Json<Vec<CdnaDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_cdna_detailed(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in crate::handlers) async fn select_cdna_detailed(
    tx: &db::Transaction<'_>,
    query: &mut CdnaQuery,
) -> Result<Vec<CdnaDetailed>, ErrorInner> {
    static SELECT_DETAILED_CDNA: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    let sql = SELECT_DETAILED_CDNA.finish_with_query(query);

    let stream = tx.query_stream(sql).await?;
    Ok(stream
        .map(|row| row.map(map_detailed_row).unwrap())
        .collect()
        .await)
}

fn map_detailed_row(row: Row) -> CdnaDetailed {
    let record: SavedCdnaRecord = row.get("cdna");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");

    CdnaDetailed {
        links: cdna_simple_links(record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
}
