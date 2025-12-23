use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use cellnoor_models::chromium_dataset::{
    ChromiumDatasetMetricsFilename, metrics::ParsedMetricsData,
};
use cellnoor_schema::chromium_dataset_metrics_files;
use diesel::prelude::*;

use crate::{
    api::{self, extract::auth::AuthenticatedUser},
    db::{self, Operation},
    state::AppState,
};

const CSV_CONTENT_TYPE: &str = "text/csv";
const JSON_CONTENT_TYPE: &[u8] = b"application/json";

#[axum::debug_handler]
pub async fn fetch_metrics_file(
    path: ChromiumDatasetMetricsFilename,
    state: State<AppState>,
    user: AuthenticatedUser,
    request: axum::extract::Request,
) -> Result<(StatusCode, Response<Body>), api::ErrorResponse> {
    tracing::info!(
        "fetching Chromium dataset metrics file {}/{}/{}",
        path.0,
        path.1,
        path.2
    );

    let db_conn = state.db_conn().await?;

    let response = db_conn
        .interact(move |db_conn| {
            let headers = request.headers();
            let content_type = headers
                .get("Accept")
                .or(headers.get("accept"))
                .map_or(JSON_CONTENT_TYPE, HeaderValue::as_bytes);

            (path, content_type).execute_as_user(user.id(), db_conn)
        })
        .await??;

    Ok((StatusCode::OK, response))
}

impl db::Operation<Response<Body>> for (ChromiumDatasetMetricsFilename, &[u8]) {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Response<Body>, db::Error> {
        let (ChromiumDatasetMetricsFilename(dataset_id, directory, filename), content_type) = self;
        let query = chromium_dataset_metrics_files::table
            .filter(chromium_dataset_metrics_files::dataset_id.eq(dataset_id))
            .filter(chromium_dataset_metrics_files::directory.eq(directory))
            .filter(chromium_dataset_metrics_files::filename.eq(filename));

        let response = if content_type == CSV_CONTENT_TYPE.as_bytes() {
            let query = query
                .filter(chromium_dataset_metrics_files::content_type.eq(CSV_CONTENT_TYPE))
                .select(chromium_dataset_metrics_files::raw_content);

            let mut response = query
                .first::<Vec<u8>>(db_conn)
                .map(Body::from)
                .map(Response::new)?;

            response
                .headers_mut()
                .insert("Content-Type", HeaderValue::from_static(CSV_CONTENT_TYPE));

            response
        } else {
            let query = query.select(chromium_dataset_metrics_files::parsed_data);
            query
                .first::<ParsedMetricsData>(db_conn)
                .map(Json)
                .map(Json::into_response)?
        };

        Ok(response)
    }
}
