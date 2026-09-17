use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    specimen::{SavedSpecimenRecordDetailed, SpecimenDetailed, SpecimenQuery},
};

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::projects::index_compact::project_from_record,
    state::AppState,
};

pub async fn index_specimens_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SpecimenQuery>,
) -> Result<Json<Vec<SpecimenDetailed>>, Error> {
    state
        .in_transaction(user, async |tx| select_specimens_detailed(tx, &query).await)
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_specimens_detailed(
    tx: &db::Transaction<'_>,
    query: &SpecimenQuery,
) -> Result<Vec<SpecimenDetailed>, ErrorInner> {
    static SELECT_DETAILED_SPECIMEN: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select(&SELECT_DETAILED_SPECIMEN, query)
        .await?
        .into_iter()
        .map(specimen_from_detailed_record)
        .collect())
}

fn specimen_from_detailed_record(
    SavedSpecimenRecordDetailed {
        specimen,
        project,
        measurements,
    }: SavedSpecimenRecordDetailed,
) -> SpecimenDetailed {
    SpecimenDetailed {
        links: SimpleLinks::from_str_and_id("/specimens", specimen.id),
        record: specimen,
        project: project_from_record(project),
        measurements,
    }
}
