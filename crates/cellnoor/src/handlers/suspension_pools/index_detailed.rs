use axum::{Json, extract::State};
use cellnoor_types::suspension_pool::{
    SavedSuspensionPoolRecord, SavedTaggedSpecimenRecord, SuspensionPoolDetailed,
    SuspensionPoolQuery,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::suspension_pools::index_compact::{
        suspension_pool_links, tagged_specimen_from_record,
    },
    state::AppState,
};

pub async fn index_suspension_pools_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SuspensionPoolQuery>,
) -> Result<Json<Vec<SuspensionPoolDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspension_pools_detailed(&tx, &mut query)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_suspension_pools_detailed(
    tx: &db::Transaction<'_>,
    query: &mut SuspensionPoolQuery,
) -> Result<Vec<SuspensionPoolDetailed>, ErrorInner> {
    static SELECT_DETAILED_SUSPENSION_POOL: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    let sql = SELECT_DETAILED_SUSPENSION_POOL.finish_with_query(query);

    let stream = tx.query_stream(sql).await?;
    Ok(stream
        .map(|row| row.map(map_detailed_row).unwrap())
        .collect()
        .await)
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
