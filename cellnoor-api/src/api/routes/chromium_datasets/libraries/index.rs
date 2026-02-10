use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, library::LibrarySummary};
use cellnoor_schema::{chromium_dataset_libraries, libraries};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_chromium_dataset_libraries(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<LibrarySummary>>, db::Error> {
    select_chromium_dataset_libraries(user.projects(), id, &db_conn)
        .await
        .map(Json)
}

async fn select_chromium_dataset_libraries(
    authorized_projects: &AuthProjects,
    dataset_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<LibrarySummary>, db::Error> {
    let query = chromium_dataset_libraries::table
        .inner_join(libraries::table)
        .select(LibrarySummary::as_select())
        .filter(chromium_dataset_libraries::dataset_id.eq(dataset_id));

    let ds_libraries = match authorized_projects {
        AuthProjects::All => query.load(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            query
                .filter(libraries::project_id.eq_any(project_ids.iter()))
                .load(&mut db_conn)
                .await?
        }
    };

    Ok(ds_libraries)
}
