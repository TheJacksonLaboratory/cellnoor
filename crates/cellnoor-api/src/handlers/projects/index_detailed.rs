use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    project::{ProjectDetailed, ProjectQuery, SavedProjectRecordDetailed},
};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_projects_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<ProjectQuery>,
) -> Result<Json<Vec<ProjectDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| select_projects_detailed(tx, &query).await)
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_projects_detailed(
    tx: &db::Transaction<'_>,
    query: &ProjectQuery,
) -> Result<Vec<ProjectDetailed>, DbError> {
    static SELECT_DETAILED_PROJECT: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select(&SELECT_DETAILED_PROJECT, query)
        .await?
        .into_iter()
        .map(project_from_detailed_record)
        .collect())
}

fn project_from_detailed_record(record: SavedProjectRecordDetailed) -> ProjectDetailed {
    ProjectDetailed {
        links: SimpleLinks::from_str_and_id("/projects", record.project.id.into()),
        record,
    }
}
