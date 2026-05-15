use axum::{Json, extract::State};
use cellnoor_types::suspension_pool::{
    SavedSuspensionPoolRecord, SavedTaggedSpecimenRecord, SuspensionPool, SuspensionPoolLinks,
    SuspensionPoolQuery, TaggedSpecimen,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, construct_select_stmt},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_suspension_pools(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SuspensionPoolQuery>,
) -> Result<Json<Vec<SuspensionPool>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspension_pools(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_suspension_pools(
    tx: &db::Transaction<'_>,
    query: &SuspensionPoolQuery,
) -> Result<Vec<SuspensionPool>, ErrorInner> {
    let pools = if query.detailed {
        let (sql, params) = construct_detailed_select_stmt(query);
        let stream = tx.query_stream(&sql, params).await?;
        stream
            .map(|row| row.map(map_detailed_row).unwrap())
            .collect()
            .await
    } else {
        let (sql, params) = construct_select_stmt(
            "suspension_pool_to_specimen",
            &["distinct on ((suspension_pool).id) suspension_pool"],
            None,
            query,
        );
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(SuspensionPool::from_record).collect().await
    };

    Ok(pools)
}

fn construct_detailed_select_stmt(
    query: &SuspensionPoolQuery,
) -> (String, Vec<&(dyn ToSql + Sync)>) {
    construct_select_stmt(
        "suspension_pool_to_specimen",
        &[
            "suspension_pool",
            "array_agg((specimen, multiplexing_tag)::tagged_specimen) as specimens",
            "array(select m from suspension_pool_measurement as m where m.pool_id = \
             (suspension_pool).id) as measurements",
            "array(select prep.prepared_by from suspension_pool_preparer as prep where \
             prep.pool_id = (suspension_pool).id) as preparers",
        ],
        Some("suspension_pool"),
        query,
    )
}

fn map_detailed_row(row: Row) -> SuspensionPool {
    let record: SavedSuspensionPoolRecord = row.get("suspension_pool");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");

    SuspensionPool::Detailed {
        links: SuspensionPoolLinks::from_id(record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(TaggedSpecimen::from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
}
