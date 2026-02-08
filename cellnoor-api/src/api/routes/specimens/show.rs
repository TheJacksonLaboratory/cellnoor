use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, specimen::Specimen};
use cellnoor_schema::specimens;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn show_specimen(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Specimen>, db::Error> {
    select_specimen_by_id(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn select_specimen_by_id(
    authorized_projects: &AuthProjects,
    specimen_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Specimen, db::Error> {
    let q = Specimen::query().filter(specimens::id.eq(specimen_id));

    let specimen = match authorized_projects {
        AuthProjects::Some { project_ids } => {
            q.filter(specimens::project_id.eq_any(project_ids.iter()))
                .first(&mut db_conn)
                .await?
        }
        AuthProjects::All => q.first(&mut db_conn).await?,
    };

    Ok(specimen)
}
