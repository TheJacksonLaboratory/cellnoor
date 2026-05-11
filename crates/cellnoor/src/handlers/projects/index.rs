use axum::{Json, extract::State};
use cellnoor_types::project::{Project, ProjectQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_projects(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<ProjectQuery>,
) -> Result<Json<Vec<Project>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_projects(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_projects(
    tx: &db::Transaction<'_>,
    query: &ProjectQuery,
) -> Result<Vec<Project>, ErrorInner> {
    let (clause, params) = query.to_sql_query();

    let projects = if query.detailed {
        let sql = format!("select project_detailed from project_detailed {clause}");
        let stream = tx.query_into_stream(&sql, params).await?;
        stream.map(Project::from_detailed_record).collect().await
    } else {
        let sql = format!("select project from project {clause}");
        let stream = tx.query_into_stream(&sql, params).await?;
        stream.map(Project::from_record).collect().await
    };

    Ok(projects)
}
