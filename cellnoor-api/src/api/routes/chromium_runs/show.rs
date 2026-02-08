use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, chromium_run::ChromiumRun};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn show_chromium_run(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<ChromiumRun>, db::Error> {
    select_chromium_run_by_id(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub(super) async fn select_chromium_run_by_id(
    authorized_projects: &AuthProjects,
    chromium_run_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<ChromiumRun, db::Error> {
    use cellnoor_schema::chromium_runs::dsl::*;

    let query = ChromiumRun::query().filter(id.eq(chromium_run_id));

    let chromium_run = match authorized_projects {
        AuthProjects::All => query.first(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            query
                .filter(project_id.eq_any(project_ids.iter()))
                .first(&mut db_conn)
                .await?
        }
    };

    Ok(chromium_run)
}
