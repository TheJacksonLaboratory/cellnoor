use axum::{Json, extract::State};
use cellnoor_types::suspension_pool::{
    SavedSuspensionPoolRecord, SavedTaggedSpecimenRecord, SuspensionPool, SuspensionPoolLinks,
    SuspensionPoolQuery, TaggedSpecimen,
};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db,
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
        let (clause, params) = query.to_sql_query_with_group_by("group by suspension_pool");
        let sql = format!("{} {clause}", include_str!("./select_detailed.sql"));
        let stream = tx.query_stream(&sql, params).await?;
        stream
            .map(|row| {
                let row = row.unwrap();
                let record: SavedSuspensionPoolRecord = row.get(0);
                let specimens: Vec<SavedTaggedSpecimenRecord> = row.get(1);

                SuspensionPool::Detailed {
                    links: SuspensionPoolLinks::from_id(record.id),
                    record,
                    specimens: specimens
                        .into_iter()
                        .map(TaggedSpecimen::from_record)
                        .collect(),
                    measurements: row.get(2),
                    preparers: row.get(3),
                }
            })
            .collect()
            .await
    } else {
        let (clause, params) = query.to_sql_query();
        let sql = format!(
            "select distinct on ((suspension_pool).id) suspension_pool from \
             suspension_pool_to_specimen {clause}"
        );
        let stream = tx.query_stream_into(&sql, params).await?;
        stream.map(SuspensionPool::from_record).collect().await
    };

    Ok(pools)
}
