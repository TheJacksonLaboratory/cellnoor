use axum::{Json, extract::State};
use cellnoor_types::suspension::{SavedSuspensionRecordDetailed, SuspensionDetailed, SuspensionQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, BaseSqlStmt},
    error::{Error, ErrorInner},
    handlers::{
        specimens::index_compact::specimen_from_record,
        suspensions::index_compact::suspension_simple_links,
    },
    state::AppState,
};

fn suspension_from_detailed_record(
    SavedSuspensionRecordDetailed {
        suspension,
        specimen,
        measurements,
        preparers,
    }: SavedSuspensionRecordDetailed,
) -> SuspensionDetailed {
    SuspensionDetailed {
        links: suspension_simple_links(suspension.id),
        record: suspension,
        specimen: specimen_from_record(specimen),
        measurements,
        preparers,
    }
}

pub async fn index_suspensions_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SuspensionQuery>,
) -> Result<Json<Vec<SuspensionDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspensions_detailed(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_suspensions_detailed(
    tx: &db::Transaction<'_>,
    query: &mut SuspensionQuery,
) -> Result<Vec<SuspensionDetailed>, ErrorInner> {
    let sql = BaseSqlStmt::new(include_str!("index/select_detailed.sql"))
        .finish_with_query(query)?;

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(suspension_from_detailed_record).collect().await)
}
