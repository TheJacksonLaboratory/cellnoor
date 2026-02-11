use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, suspension_pool::measurement::SuspensionPoolMeasurement};
use cellnoor_schema::{suspension_pool_measurements, suspension_pools};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_measurements(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<SuspensionPoolMeasurement>>, db::Error> {
    select_suspension_pool_measurements(user.projects(), id, &db_conn)
        .await
        .map(Json)
}

pub async fn select_suspension_pool_measurements(
    authorized_projects: &AuthProjects,
    suspension_pool_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SuspensionPoolMeasurement>, db::Error> {
    let stmt = SuspensionPoolMeasurement::query()
        .filter(suspension_pool_measurements::pool_id.eq(suspension_pool_id));

    let measurements = match authorized_projects {
        AuthProjects::All => stmt.load(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            stmt.inner_join(suspension_pools::table)
                .filter(suspension_pools::project_id.eq_any(project_ids.iter()))
                .load(&mut db_conn)
                .await?
        }
    };

    Ok(measurements)
}
