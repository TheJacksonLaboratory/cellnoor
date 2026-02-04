use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, cdna::Cdna};
use cellnoor_schema::cdna;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn show_cdna(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Cdna>, db::Error> {
    select_cdna_by_id(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn select_cdna_by_id(
    authorized_projects: &AuthProjects,
    cdna_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Cdna, db::Error> {
    let stmt = Cdna::query().filter(cdna::id.eq(cdna_id));

    let cdna = match authorized_projects {
        AuthProjects::All => stmt.first(&mut db_conn).await?,
        AuthProjects::Restricted(projects) => {
            stmt.filter(cdna::project_id.eq_any(projects.iter()))
                .first(&mut db_conn)
                .await?
        }
    };

    Ok(cdna)
}
