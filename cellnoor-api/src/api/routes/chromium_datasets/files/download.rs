use std::str::FromStr;

use axum::{
    Extension,
    extract::{Path, State},
    http::HeaderValue,
};
use axum_extra::TypedHeader;
use cellnoor_models::chromium_dataset::metrics::ParsedMetricsData;
use cellnoor_schema::{chromium_dataset_files, chromium_datasets};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use headers::ContentType;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::auth::{AuthProjects, AuthUser},
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection},
    state::AppState,
};

pub const ANY_CONTENT_TYPE: &str = "*/*";
pub const CSV_CONTENT_TYPE: &str = "text/csv";
pub const HTML_CONTENT_TYPE: &str = "text/html";
pub const JSON_CONTENT_TYPE: &str = "application/json";

#[derive(Deserialize, JsonSchema)]
#[schemars(inline)]
pub struct FilePath {
    // This field has to be called `id` instead of `dataset_id` because there are other routes that
    // depend on the struct `IdParameter`, whose only field is called `id`
    pub id: Uuid,
    pub path: String,
}

type Response = (TypedHeader<ContentType>, Vec<u8>);

#[axum::debug_handler]
pub async fn download_chromium_dataset_file(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(file_path): Path<FilePath>,
    request: axum::extract::Request,
) -> Result<Response, db::Error> {
    tracing::info!(
        "fetching Chromium dataset file with path {}",
        file_path.path
    );

    let headers = request.headers();
    let accept = headers
        .get("Accept")
        .or(headers.get("accept"))
        .map(HeaderValue::to_str)
        .and_then(Result::ok);

    let response =
        select_chromium_dataset_file(&file_path, user.projects(), accept, &db_conn).await?;

    Ok(response)
}

async fn select_chromium_dataset_file(
    file_path: &FilePath,
    authorized_projects: &AuthProjects,
    accept: Option<&str>,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Response, db::Error> {
    let filter = chromium_dataset_files_filter(file_path, authorized_projects, accept);

    let query = chromium_dataset_files::table
        .inner_join(chromium_datasets::table)
        .filter(filter);

    let response = if return_raw_content(accept) {
        let (content_type, raw_content) = query
            .select((
                chromium_dataset_files::content_type,
                chromium_dataset_files::raw_content,
            ))
            .first::<(String, Vec<u8>)>(&mut db_conn)
            .await?;

        let content_type = TypedHeader(ContentType::from_str(&content_type).unwrap());

        (content_type, raw_content)
    } else {
        let content = query
            .select(chromium_dataset_files::parsed_data)
            .first::<Option<ParsedMetricsData>>(&mut db_conn)
            .await
            .map(|d| serde_json::to_vec(&d))?
            .unwrap();

        (TypedHeader(ContentType::json()), content)
    };

    Ok(response)
}

fn chromium_dataset_files_filter<'a, QS: 'a>(
    FilePath { id, path }: &'a FilePath,
    authorized_projects: &'a AuthProjects,
    accept: Option<&'a str>,
) -> BoxedFilter<'a, QS>
where
    chromium_dataset_files::dataset_id: SelectableExpression<QS>,
    chromium_dataset_files::path: SelectableExpression<QS>,
    chromium_datasets::project_id: SelectableExpression<QS>,
    chromium_dataset_files::content_type: SelectableExpression<QS>,
{
    let mut filter = BoxedFilter::new_true()
        .and_condition(chromium_dataset_files::dataset_id.eq(id))
        .and_condition(chromium_dataset_files::path.eq(path));

    if let AuthProjects::Some { project_ids } = authorized_projects {
        filter = filter.and_condition(chromium_datasets::project_id.eq_any(project_ids.iter()));
    }

    let Some(content_type) = accept else {
        return filter;
    };

    if content_type.contains(ANY_CONTENT_TYPE) {
        return filter;
    }

    let mut content_type_filter = BoxedFilter::new_false();

    if content_type.contains(CSV_CONTENT_TYPE) {
        content_type_filter = content_type_filter
            .or_condition(chromium_dataset_files::content_type.eq(CSV_CONTENT_TYPE));
    }

    if content_type.contains(HTML_CONTENT_TYPE) {
        content_type_filter = content_type_filter
            .or_condition(chromium_dataset_files::content_type.eq(HTML_CONTENT_TYPE));
    }

    // If the client asks for JSON, we can return files that were originally JSON or
    // files that were originally CSV and parsed into JSON
    if content_type.contains(JSON_CONTENT_TYPE) {
        content_type_filter = content_type_filter.or_condition(
            chromium_dataset_files::content_type.eq_any([CSV_CONTENT_TYPE, JSON_CONTENT_TYPE]),
        );
    }

    filter.and_condition(content_type_filter)
}

fn return_raw_content(maybe_content_type: Option<&str>) -> bool {
    // This is a terrible, standards-noncompliant implementation, but for our
    // purposes its fine
    match maybe_content_type {
        None => true,
        Some(content_type) if content_type.contains(JSON_CONTENT_TYPE) => false,
        Some(_) => true,
    }
}
