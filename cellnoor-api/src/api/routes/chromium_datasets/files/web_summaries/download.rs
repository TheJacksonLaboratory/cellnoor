use axum::{
    Extension,
    extract::{Path, State},
    response::Html,
};
use cellnoor_schema::{chromium_dataset_web_summaries, chromium_datasets};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    api::{
        auth::{AuthProjects, AuthUser},
        routes::chromium_datasets::files::common::FilePath,
    },
    db::{self, DbConnection},
    state::AppState,
};

pub async fn download_web_summary(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(web_summary_path): Path<FilePath>,
) -> Result<Html<Vec<u8>>, db::Error> {
    tracing::info!(
        "fetching web summary {}/{} for Chromium dataset {}",
        web_summary_path.directory,
        web_summary_path.filename,
        web_summary_path.id
    );

    let file =
        select_chromium_dataset_web_summaries(user.projects(), &web_summary_path, &db_conn).await?;

    Ok(Html(file))
}

async fn select_chromium_dataset_web_summaries(
    authorized_projects: &AuthProjects,
    FilePath {
        id,
        directory,
        filename,
    }: &FilePath,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<u8>, db::Error> {
    let query = chromium_dataset_web_summaries::table
        .select(chromium_dataset_web_summaries::content)
        .filter(chromium_dataset_web_summaries::dataset_id.eq(id))
        .filter(chromium_dataset_web_summaries::directory.eq(directory))
        .filter(chromium_dataset_web_summaries::filename.eq(filename));

    let web_summary = match authorized_projects {
        AuthProjects::All => query.first(&mut db_conn).await?,
        AuthProjects::Some { project_ids } => {
            query
                .inner_join(chromium_datasets::table)
                .filter(chromium_datasets::project_id.eq_any(project_ids.iter()))
                .first(&mut db_conn)
                .await?
        }
    };

    Ok(web_summary)
}
