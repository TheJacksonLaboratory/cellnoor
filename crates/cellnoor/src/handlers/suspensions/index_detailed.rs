use axum::{Json, extract::State};
use cellnoor_types::suspension::{
    SavedSuspensionRecordDetailed, SuspensionDetailed, SuspensionQuery,
};

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::{
        specimens::index_compact::specimen_from_record,
        suspensions::index_compact::suspension_simple_links,
    },
    state::AppState,
};

pub async fn index_suspensions_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SuspensionQuery>,
) -> Result<Json<Vec<SuspensionDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspensions_detailed(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_suspensions_detailed(
    tx: &db::Transaction<'_>,
    query: &SuspensionQuery,
) -> Result<Vec<SuspensionDetailed>, ErrorInner> {
    static SELECT_DETAILED_SUSPENSION: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select(&SELECT_DETAILED_SUSPENSION, query)
        .await?
        .into_iter()
        .map(suspension_from_detailed_record)
        .collect())
}

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
