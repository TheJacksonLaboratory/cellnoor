use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use cellnoor_models::{IdParameter, cdna::measurement::CdnaMeasurement};
use cellnoor_schema::{cdna, cdna_measurements};
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
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<CdnaMeasurement>>, db::Error> {
    select_measurements(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn select_measurements(
    authorized_projects: &AuthProjects,
    cdna_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<Vec<CdnaMeasurement>, db::Error> {
    let query = CdnaMeasurement::query()
        .order_by(cdna_measurements::measured_at)
        .filter(cdna_measurements::cdna_id.eq(cdna_id));

    let measurements = match authorized_projects {
        AuthProjects::All => query.load(db_conn).await?,
        AuthProjects::Restricted(projects) => {
            query
                .inner_join(cdna::table)
                .filter(cdna::project_id.eq_any(projects.iter()))
                .load(db_conn)
                .await?
        }
    };

    Ok(measurements)
}
