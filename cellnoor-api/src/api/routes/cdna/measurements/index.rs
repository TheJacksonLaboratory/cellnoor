use axum::{
    Extension, Json,
    extract::{Path, State},
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

pub async fn index_cdna_measurements(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Vec<CdnaMeasurement>>, db::Error> {
    select_cdna_measurements(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn select_cdna_measurements(
    authorized_projects: &AuthProjects,
    cdna_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<CdnaMeasurement>, db::Error> {
    let query = CdnaMeasurement::query()
        .order_by(cdna_measurements::measured_at)
        .filter(cdna_measurements::cdna_id.eq(cdna_id));

    let measurements = match authorized_projects {
        AuthProjects::All => query.load(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            query
                .inner_join(cdna::table)
                .filter(cdna::project_id.eq_any(project_ids.iter()))
                .load(&mut db_conn)
                .await?
        }
    };

    Ok(measurements)
}
