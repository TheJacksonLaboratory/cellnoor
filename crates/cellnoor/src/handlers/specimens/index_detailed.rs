use axum::{Json, extract::State};
use cellnoor_types::specimen::{SavedSpecimenRecordDetailed, SpecimenDetailed, SpecimenQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, BaseSqlStmt},
    error::{Error, ErrorInner},
    handlers::{
        projects::index_compact::project_from_record,
        specimens::index_compact::specimen_simple_links,
    },
    state::AppState,
};

pub async fn index_specimens_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SpecimenQuery>,
) -> Result<Json<Vec<SpecimenDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_specimens_detailed(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// This visibility is necessary for RLS tests
pub(in crate::handlers) async fn select_specimens_detailed(
    tx: &db::Transaction<'_>,
    query: &mut SpecimenQuery,
) -> Result<Vec<SpecimenDetailed>, ErrorInner> {
    let sql =
        BaseSqlStmt::new(include_str!("index/select_detailed.sql")).finish_with_query(query)?;

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(specimen_from_detailed_record).collect().await)
}

fn specimen_from_detailed_record(
    SavedSpecimenRecordDetailed {
        specimen,
        project,
        measurements,
    }: SavedSpecimenRecordDetailed,
) -> SpecimenDetailed {
    SpecimenDetailed {
        links: specimen_simple_links(specimen.id),
        record: specimen,
        project: project_from_record(project),
        measurements,
    }
}
