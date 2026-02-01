use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
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
    todo!()
}

pub(super) async fn select_chromium_run_by_id(
    authorized_projects: &AuthProjects,
    chromium_run_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<ChromiumRun, db::Error> {
    use cellnoor_schema::chromium_runs::dsl::*;

    let query = ChromiumRun::query().filter(id.eq(chromium_run_id));

    match
}
