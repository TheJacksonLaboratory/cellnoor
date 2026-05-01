use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, library::measurement::LibraryMeasurement};
use cellnoor_schema::{libraries, library_measurements};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_library_measurements(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<LibraryMeasurement>>, db::Error> {
    select_library_measurements(user.projects(), id, &db_conn)
        .await
        .map(Json)
}

pub async fn select_library_measurements(
    authorized_projects: &AuthProjects,
    library_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<LibraryMeasurement>, db::Error> {
    let query = LibraryMeasurement::query()
        .order_by(library_measurements::measured_at)
        .filter(library_measurements::library_id.eq(library_id));

    let measurements = match authorized_projects {
        AuthProjects::All => query.load(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            query
                .inner_join(libraries::table)
                .filter(libraries::project_id.eq_any(project_ids.iter()))
                .load(&mut db_conn)
                .await?
        }
    };

    Ok(measurements)
}
