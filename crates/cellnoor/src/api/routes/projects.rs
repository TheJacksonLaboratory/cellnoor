use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::project::{ProjectCompact, ProjectQuery, SimpleProjectQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::projects::{
        create::create_project, delete::delete_project, index_compact::index_projects,
        index_detailed::index_projects_detailed, show::show_project, update::update_project,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_project).get(index_projects_simple))
        .api_route("/search", post(index_projects))
        .api_route("/search/detailed", post(index_projects_detailed))
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
) -> Result<Json<Vec<ProjectCompact>>, Error> {
    index_projects(state, user, Json(ProjectQuery::from_simple_query(q))).await
}
