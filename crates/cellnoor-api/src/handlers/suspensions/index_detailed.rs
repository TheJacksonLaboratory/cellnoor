use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    suspension::{SavedSuspensionRecordDetailed, SuspensionDetailed, SuspensionQuery},
};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    handlers::specimens::index_compact::specimen_from_record,
    state::AppState,
};

pub async fn index_suspensions_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SuspensionQuery>,
) -> Result<Json<Vec<SuspensionDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| {
            select_suspensions_detailed(tx, &query).await
        })
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_suspensions_detailed(
    tx: &db::Transaction<'_>,
    query: &SuspensionQuery,
) -> Result<Vec<SuspensionDetailed>, DbError> {
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
        links: SimpleLinks::from_str_and_id("/suspensions", suspension.id),
        record: suspension,
        specimen: specimen_from_record(specimen),
        measurements,
        preparers,
    }
}
