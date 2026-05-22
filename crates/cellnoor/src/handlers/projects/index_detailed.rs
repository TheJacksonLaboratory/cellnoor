use axum::{Json, extract::State};
use cellnoor_types::project::{ProjectDetailed, ProjectQuery, SavedProjectRecordDetailed};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, BaseSqlStmt},
    error::{Error, ErrorInner},
    handlers::projects::index_compact::project_simple_links,
    state::AppState,
};

pub async fn index_projects_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ProjectQuery>,
) -> Result<Json<Vec<ProjectDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_projects_detailed(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// This visibility is necessary for RLS tests
pub(in crate::handlers) async fn select_projects_detailed(
    tx: &db::Transaction<'_>,
    query: &mut ProjectQuery,
) -> Result<Vec<ProjectDetailed>, ErrorInner> {
    let sql =
        BaseSqlStmt::new(include_str!("index/select_detailed.sql")).finish_with_query(query)?;

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(project_from_detailed_record).collect().await)
}

fn project_from_detailed_record(record: SavedProjectRecordDetailed) -> ProjectDetailed {
    ProjectDetailed {
        links: project_simple_links(record.project.id),
        record,
    }
}
