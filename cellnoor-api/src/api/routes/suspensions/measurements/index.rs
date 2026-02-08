use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter, specimen::measurement::SpecimenMeasurement,
    suspension::measurement::SuspensionMeasurement,
};
use cellnoor_schema::{specimen_measurements, specimens, suspension_measurements, suspensions};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_suspension_measurements(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<SuspensionMeasurement>>, db::Error> {
    let authorized_projects = user.projects();

    select_suspension_measurements(authorized_projects, id, &mut db_conn)
        .await
        .map(Json)
}

async fn select_suspension_measurements(
    authorized_projects: &AuthProjects,
    suspension_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SuspensionMeasurement>, db::Error> {
    let q = SuspensionMeasurement::query()
        .order_by(suspension_measurements::measured_at)
        .filter(suspension_measurements::suspension_id.eq(suspension_id));

    let measurements = match authorized_projects {
        AuthProjects::All => q.load(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            q.inner_join(suspensions::table)
                .filter(suspensions::project_id.eq_any(project_ids.iter()))
                .load(&mut db_conn)
                .await?
        }
    };

    Ok(measurements)
}
