use std::collections::HashSet;

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
    api::auth::AuthenticatedUser,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_specimen_measurements(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthenticatedUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<SpecimenMeasurement>>, db::Error> {
    let authorized_projects = user.authorized_projects();

    select_specimen_measurements(authorized_projects, id, &mut db_conn)
        .await
        .map(Json)
}

async fn select_specimen_measurements(
    authorized_projects: Option<&HashSet<Uuid>>,
    specimen_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<Vec<SpecimenMeasurement>, db::Error> {
    let q = SpecimenMeasurement::query()
        .order_by(specimen_measurements::measured_at)
        .filter(specimen_measurements::specimen_id.eq(specimen_id));

    let measurements = if let Some(projects) = authorized_projects {
        q.inner_join(specimens::table)
            .filter(specimens::project_id.eq_any(projects))
            .load(db_conn)
            .await?
    } else {
        q.load(db_conn).await?
    };

    Ok(measurements)
}
