use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, suspension_pool::SuspensionPool};
use cellnoor_schema::suspension_pools;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn show_suspension_pool(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<SuspensionPool>, db::Error> {
    select_suspension_pool_by_id(user.projects(), id, &mut db_conn)
        .await
        .map(Json)
}

pub(super) async fn select_suspension_pool_by_id(
    authorized_projects: &AuthProjects,
    suspension_pool_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<SuspensionPool, db::Error> {
    let stmt = SuspensionPool::query().filter(suspension_pools::id.eq(suspension_pool_id));

    let suspension_pool = match authorized_projects {
        AuthProjects::All => stmt.first(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            stmt.filter(suspension_pools::project_id.eq_any(project_ids.iter()))
                .first(&mut db_conn)
                .await?
        }
    };

    Ok(suspension_pool)
}
