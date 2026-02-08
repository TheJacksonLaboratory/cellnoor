use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, library::Library};
use cellnoor_schema::libraries;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn show_library(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Library>, db::Error> {
    select_library_by_id(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn select_library_by_id(
    authorized_projects: &AuthProjects,
    library_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Library, db::Error> {
    let query = Library::query().filter(libraries::id.eq(library_id));

    let library = match authorized_projects {
        AuthProjects::All => query.first(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            query
                .filter(libraries::project_id.eq_any(project_ids.iter()))
                .first(&mut db_conn)
                .await?
        }
    };

    Ok(library)
}
