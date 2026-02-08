use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, suspension::Suspension};
use cellnoor_schema::suspensions;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn show_suspension(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Suspension>, db::Error> {
    select_suspension_by_id(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub(super) async fn select_suspension_by_id(
    authorized_projects: &AuthProjects,
    suspension_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Suspension, db::Error> {
    let stmt = Suspension::query().filter(suspensions::id.eq(suspension_id));

    let suspension = match authorized_projects {
        AuthProjects::All => stmt.first(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            stmt.filter(suspensions::project_id.eq_any(project_ids.iter()))
                .first(&mut db_conn)
                .await?
        }
    };

    Ok(suspension)
}
