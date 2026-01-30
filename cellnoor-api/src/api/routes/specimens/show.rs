use std::collections::HashSet;

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
    api::auth::AuthenticatedUser,
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn show_specimen(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthenticatedUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Specimen>, db::Error> {
    let projects = (!user.is_admin()).then(|| user.projects());

    select_specimen_by_id(projects, id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn select_specimen_by_id(
    authorized_projects: Option<&HashSet<Uuid>>,
    specimen_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<Specimen, db::Error> {
    let q = Specimen::query().filter(specimens::id.eq(specimen_id));

    let specimen = match authorized_projects {
        Some(projects) => {
            q.filter(specimens::project_id.eq_any(projects))
                .first(db_conn)
                .await?
        }
        None => q.first(db_conn).await?,
    };

    Ok(specimen)
}
