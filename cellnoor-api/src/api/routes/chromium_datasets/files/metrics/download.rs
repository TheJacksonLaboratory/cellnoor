use axum::{
    Extension, Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Response},
    response::IntoResponse,
};
use cellnoor_models::chromium_dataset::metrics::ParsedMetricsData;
use cellnoor_schema::{chromium_dataset_metrics_files, chromium_datasets};
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

const CSV_CONTENT_TYPE: &str = "text/csv";
const JSON_CONTENT_TYPE: &[u8] = b"application/json";

pub async fn download_metrics_file(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(file_path): Path<FilePath>,
    request: axum::extract::Request,
) -> Result<Response<Body>, db::Error> {
    tracing::info!(
        "fetching Chromium dataset metrics file {}/{}/{}",
        file_path.dataset_id,
        file_path.directory,
        file_path.filename
    );

    let headers = request.headers();
    let content_type = headers
        .get("Accept")
        .or(headers.get("accept"))
        .map_or(JSON_CONTENT_TYPE, HeaderValue::as_bytes);

    let response = select_chromium_dataset_metrics_by_id(
        user.projects(),
        content_type,
        &file_path,
        &mut db_conn,
    )
    .await?;

    Ok(response)
}

async fn select_chromium_dataset_metrics_by_id(
    authorized_projects: &AuthProjects,
    content_type: &[u8],
    FilePath {
        dataset_id,
        directory,
        filename,
    }: &FilePath,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Response<Body>, db::Error> {
    let query = chromium_dataset_metrics_files::table
        .filter(chromium_dataset_metrics_files::dataset_id.eq(dataset_id))
        .filter(chromium_dataset_metrics_files::directory.eq(directory))
        .filter(chromium_dataset_metrics_files::filename.eq(filename));

    let response = if content_type == CSV_CONTENT_TYPE.as_bytes() {
        let query = query
            .filter(chromium_dataset_metrics_files::content_type.eq(CSV_CONTENT_TYPE))
            .select(chromium_dataset_metrics_files::raw_content);

        let response = match authorized_projects {
            AuthProjects::All => query.first::<Vec<u8>>(&mut db_conn).await,
            AuthProjects::Some { project_ids } => {
                query
                    .inner_join(chromium_datasets::table)
                    .filter(chromium_datasets::project_id.eq_any({ project_ids }.iter()))
                    .first::<Vec<u8>>(&mut db_conn)
                    .await
            }
        };

        let mut response = response.map(Body::from).map(Response::new)?;

        response
            .headers_mut()
            .insert("Content-Type", HeaderValue::from_static(CSV_CONTENT_TYPE));

        response
    } else {
        let query = query.select(chromium_dataset_metrics_files::parsed_data);
        query
            .first::<ParsedMetricsData>(&mut db_conn)
            .await
            .map(Json)
            .map(Json::into_response)?
    };

    Ok(response)
}
