use axum::{Json, extract::State};
use cellnoor_types::project::{ProjectDetailed, ProjectQuery, SavedProjectRecordDetailed};

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::projects::index_compact::project_simple_links,
    state::AppState,
};

pub async fn index_projects_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<ProjectQuery>,
) -> Result<Json<Vec<ProjectDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_projects_detailed(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_projects_detailed(
    tx: &db::Transaction<'_>,
    query: &ProjectQuery,
) -> Result<Vec<ProjectDetailed>, ErrorInner> {
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
        links: project_simple_links(record.project.id),
        record,
    }
}
