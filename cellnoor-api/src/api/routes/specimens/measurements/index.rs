use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, specimen::measurement::SpecimenMeasurement};
use cellnoor_schema::{specimen_measurements, specimens};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_specimen_measurements(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<SpecimenMeasurement>>, db::Error> {
    let authorized_projects = user.projects();

    select_specimen_measurements(authorized_projects, id, &db_conn)
        .await
        .map(Json)
}

async fn select_specimen_measurements(
    authorized_projects: &AuthProjects,
    specimen_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SpecimenMeasurement>, db::Error> {
    let q = SpecimenMeasurement::query()
        .order_by(specimen_measurements::measured_at)
        .filter(specimen_measurements::specimen_id.eq(specimen_id));

    let measurements = match authorized_projects {
        AuthProjects::All => q.load(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            q.inner_join(specimens::table)
                .filter(specimens::project_id.eq_any(project_ids.iter()))
                .load(&mut db_conn)
                .await?
        }
    };

    Ok(measurements)
}
