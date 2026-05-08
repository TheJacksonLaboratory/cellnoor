use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::project::{Project, ProjectQuery, SimpleProjectQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::projects::{
        create::create_project, delete::delete_project, index::index_projects, show::show_project,
        update::update_project,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_project).get(index_projects_simple))
        .api_route("/search", post(index_projects))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(show_project).put(update_project).delete(delete_project),
    )
}

async fn index_projects_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleProjectQuery>,
) -> Result<Json<Vec<Project>>, Error> {
    index_projects(state, user, Json(ProjectQuery::from_simple_query(q))).await
}
