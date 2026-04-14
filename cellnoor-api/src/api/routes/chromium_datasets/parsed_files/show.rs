use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_schema::{chromium_dataset_parsed_files, chromium_datasets};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    api::{
        auth::{AuthProjects, AuthUser},
        routes::chromium_datasets::{FilePath, ParsedChromiumDatasetFile},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection},
    state::AppState,
};

pub async fn show_parsed_chromium_dataset_file(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(file_path): Path<FilePath>,
) -> Result<Json<ParsedChromiumDatasetFile>, db::Error> {
    tracing::info!(
        "fetching Chromium dataset file with path {}",
        file_path.path
    );

    select_parsed_chromium_dataset_file(&file_path, user.projects(), &db_conn)
        .await
        .map(Json)
}

async fn select_parsed_chromium_dataset_file(
    file_path: &FilePath,
    authorized_projects: &AuthProjects,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<ParsedChromiumDatasetFile, db::Error> {
    let filter = chromium_dataset_raw_files_filter(file_path, authorized_projects);

    Ok(ParsedChromiumDatasetFile::query()
        .inner_join(chromium_datasets::table)
        .filter(filter)
        .first(&mut db_conn)
        .await?)
}

fn chromium_dataset_raw_files_filter<'a, QS: 'a>(
    FilePath { id, path }: &'a FilePath,
    authorized_projects: &'a AuthProjects,
) -> BoxedFilter<'a, QS>
where
    chromium_dataset_parsed_files::dataset_id: SelectableExpression<QS>,
    chromium_dataset_parsed_files::path: SelectableExpression<QS>,
    chromium_datasets::project_id: SelectableExpression<QS>,
{
    let mut filter = BoxedFilter::new_true()
        .and_condition(chromium_dataset_parsed_files::dataset_id.eq(id))
        .and_condition(chromium_dataset_parsed_files::path.eq(path));

    if let AuthProjects::Some { project_ids } = authorized_projects {
        filter = filter.and_condition(chromium_datasets::project_id.eq_any(project_ids.iter()));
    }

    filter
}
