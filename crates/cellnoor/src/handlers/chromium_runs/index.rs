use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_run::{
        ChromiumRun, ChromiumRunField, ChromiumRunQuery, SavedChromiumRunRecord,
        SavedChromiumRunRecordDetailed, SavedGemPoolWithSpecimensRecord,
    },
    order_by::OrderBy,
    tenx_assay::TenxAssay,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, SqlTemplate},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_chromium_runs(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRun>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_runs(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_chromium_runs(
    tx: &db::Transaction<'_>,
    query: &mut ChromiumRunQuery,
) -> Result<Vec<ChromiumRun>, ErrorInner> {
    // The first column in the `order by` clause needs to match the `distinct on`
    // clause
    let distinct_on = OrderBy {
        field: ChromiumRunField::Id,
        desc: true,
    };

    query.order_by.push_front(distinct_on);

    let base_stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = SqlTemplate::new(base_stmt).finish_with_query(query)?;

    let runs = if query.detailed {
        let stream = tx.query_stream(sql).await?;
        stream
            .map(|row| row.map(map_detailed_row).unwrap())
            .collect()
            .await
    } else {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(ChromiumRun::from_record).collect().await
    };

    Ok(runs)
}

fn map_detailed_row(row: Row) -> ChromiumRun {
    let chromium_run: SavedChromiumRunRecord = row.get("chromium_run");
    let assay: TenxAssay = row.get("tenx_assay");
    let gem_pools: Vec<SavedGemPoolWithSpecimensRecord> = row.get("gem_pools");

    ChromiumRun::from_detailed_record_and_gem_pools(
        SavedChromiumRunRecordDetailed {
            chromium_run,
            assay,
        },
        gem_pools,
    )
}
