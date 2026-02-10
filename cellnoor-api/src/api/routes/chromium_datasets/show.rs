use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, chromium_dataset::ChromiumDataset};
use cellnoor_schema::{
    cdna, chromium_dataset_libraries, chromium_datasets, chromium_runs, gem_pools, libraries,
    projects, tenx_assays,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn show_chromium_dataset(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<ChromiumDataset>, db::Error> {
    select_chromium_dataset_by_id(user.projects(), id, &db_conn)
        .await
        .map(Json)
}

pub(super) async fn select_chromium_dataset_by_id(
    authorized_projects: &AuthProjects,
    dataset_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<ChromiumDataset, db::Error> {
    let query = chromium_datasets_to_assay()
        .select(ChromiumDataset::as_select())
        .filter(chromium_datasets::id.eq(dataset_id));

    let dataset = match authorized_projects {
        AuthProjects::All => query.first(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            query
                .filter(chromium_datasets::project_id.eq_any(project_ids.iter()))
                .first(&mut db_conn)
                .await?
        }
    };

    Ok(dataset)
}

#[diesel::dsl::auto_type]
fn chromium_datasets_to_assay() -> _ {
    chromium_datasets::table
        .inner_join(projects::table)
        .inner_join(
            chromium_dataset_libraries::table.inner_join(
                libraries::table.inner_join(
                    cdna::table.inner_join(
                        gem_pools::table
                            .inner_join(chromium_runs::table.inner_join(tenx_assays::table)),
                    ),
                ),
            ),
        )
}
