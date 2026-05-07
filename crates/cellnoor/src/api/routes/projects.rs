use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleQuery,
    project::{Project, ProjectQuery, ProjectSortField},
};
use serde_qs::web::QsQuery;

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::projects::{create::create_project, index::index_projects, show::show_project},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_project).get(index_projects_simple))
        .api_route("/search", post(index_projects))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_project))
}

async fn index_projects_simple(
    state: State<AppState>,
    user: AuthUser,
    QsQuery(q): QsQuery<SimpleQuery<ProjectSortField>>,
) -> Result<Json<Vec<Project>>, Error> {
    index_projects(state, user, Json(ProjectQuery::from_simple_query(q))).await
}
