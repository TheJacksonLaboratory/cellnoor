use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
#[cfg(any(test, feature = "dev"))]
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, FromRequest},
    http::{HeaderMap, HeaderValue, Request},
};
use axum::{
    extract::{Multipart, Path, State, multipart::MultipartError},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use camino::Utf8Path;
use cellnoor_types::nonempty::NonemptyString;
#[cfg(any(test, feature = "dev"))]
use reqwest::{
    header::CONTENT_TYPE,
    multipart::{Form, Part},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Pool, Sql},
    error::error_response,
    handlers::{
        IdParam,
        chromium_datasets::upload_files::{
            allowed_content_type::AllowedContentType,
            upload_path::UploadPath,
            write_to_db::write_files_to_db,
            write_to_disk::{
                chromium_dataset_dir_path, compress_and_link, project_dir_paths,
                write_csv_or_json_to_disk, write_html_to_disk,
            },
        },
        specific_error_inferred_early_responses,
    },
    state::AppState,
};

mod allowed_content_type;
mod error;
mod upload_path;
mod write_to_db;
mod write_to_disk;

pub async fn upload_files(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: dataset_id }): Path<IdParam>,
    files: Multipart,
) -> Result<(), UploadFilesError> {
    write_files_to_disk_and_db(
        &app_state.db_pool,
        &app_state.static_files_dir,
        user,
        dataset_id,
        files,
    )
    .await
}

// The actual implementation lives in this separate function so we can call this
// in tests without needing a full AppState
async fn write_files_to_disk_and_db(
    db_pool: &Pool,
    static_files_dir: &Utf8Path,
    user: AuthUser,
    dataset_id: Uuid,
    mut files: Multipart,
) -> Result<(), UploadFilesError> {
    let DatasetWithProjectNames {
        dataset_name,
        project_names,
    } = {
        let mut client = db_pool.get(user).await.map_err(DbError::from)?;
        let tx = client.begin().await.map_err(DbError::from)?;

        fetch_dataset_and_project_names(&tx, &dataset_id).await?
    };

    let chromium_dataset_dir = chromium_dataset_dir_path(static_files_dir, dataset_id);
    let project_dirs = project_dir_paths(static_files_dir, &project_names, &dataset_name);

    let mut written_files = Vec::new();

    while let Some(mut field) = files.next_field().await? {
        let content_type = AllowedContentType::from_multipart_form_field(&field)?;
        let path = UploadPath::new(field.file_name())?;

        let dest = chromium_dataset_dir.join(path.as_str());
        tokio::fs::create_dir_all(dest.parent().unwrap()).await?;

        let parsed_file = match content_type {
            AllowedContentType::Html => {
                write_html_to_disk(&mut field, &dest).await?;

                None
            }
            AllowedContentType::Csv | AllowedContentType::Json => {
                write_csv_or_json_to_disk(content_type, &field.bytes().await?, &dest).await?
            }
        };

        let link_paths: Vec<_> = project_dirs.iter().map(|d| d.join(path.as_str())).collect();
        tokio::task::spawn_blocking(move || compress_and_link(&dest, &link_paths))
            .await
            .map_err(std::io::Error::other)??;

        written_files.push((path, parsed_file));
    }

    let mut client = db_pool.get(user).await.map_err(DbError::from)?;
    let tx = client.begin().await.map_err(DbError::from)?;

    write_files_to_db(&tx, dataset_id, &written_files).await?;

    tx.commit().await.map_err(DbError::from)?;

    Ok(())
}

pub(super) struct DatasetWithProjectNames {
    pub dataset_name: NonemptyString,
    pub project_names: Vec<NonemptyString>,
}

pub(super) async fn fetch_dataset_and_project_names(
    tx: &db::Transaction<'_>,
    dataset_id: &Uuid,
) -> Result<DatasetWithProjectNames, DbError> {
    static SELECT_PROJECT_NAMES: &str = include_str!("upload_files/select_project_names.sql");

    let sql = Sql::new(SELECT_PROJECT_NAMES, vec![dataset_id]);

    tx.query_one(&sql).await.map(|r| DatasetWithProjectNames {
        dataset_name: r.get("dataset_name"),
        project_names: r.get("project_names"),
    })
}

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UploadFilesError {
    #[error("failed to read multipart form: {message}")]
    Multipart { message: String },
    #[error("{message}")]
    ContentType { message: String },
    #[error("{message}")]
    Filename { message: String },
    #[error("failed to parse uploaded file: {message}")]
    Parse { message: String },
    #[error("failed to write file to disk: {message}")]
    Write { message: String },
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl UploadFilesError {
    fn status(&self) -> StatusCode {
        match self {
            Self::Multipart { .. }
            | Self::ContentType { .. }
            | Self::Filename { .. }
            | Self::Parse { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Db(e) => e.status(),
            Self::Write { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<MultipartError> for UploadFilesError {
    fn from(value: MultipartError) -> Self {
        Self::Multipart {
            message: value.to_string(),
        }
    }
}

impl From<std::io::Error> for UploadFilesError {
    fn from(value: std::io::Error) -> Self {
        Self::Write {
            message: value.to_string(),
        }
    }
}

impl IntoResponse for UploadFilesError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for UploadFilesError {
    type Inner = Self;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        specific_error_inferred_early_responses::<Self>(ctx, operation)
    }
}

#[cfg(any(test, feature = "dev"))]
pub async fn write_test_dataset_files(
    db_pool: &Pool,
    dataset_id: Uuid,
    static_files_dir: &Utf8Path,
) {
    const MULTI_METRICS: &[u8] = include_bytes!(
        "upload_files/test-data/\
         4plex_rat_liver_kidney_lgintest_testis_CF_RatLiver_A-A03_metrics_summary.csv"
    );

    let csv_header: HeaderMap = [(CONTENT_TYPE, HeaderValue::from_static("text/csv"))]
        .into_iter()
        .collect();

    let body = Form::new()
        .part(
            "web_summary",
            Part::bytes(include_bytes!(
                "upload_files/test-data/\
                 4plex_rat_liver_kidney_lgintest_testis_CF_RatLiver_A-A03_web_summary.html"
            ))
            .headers(
                [(CONTENT_TYPE, HeaderValue::from_static("text/html"))]
                    .into_iter()
                    .collect(),
            )
            .file_name("per_sample_outs/sample1/web_summary.html"),
        )
        .part(
            "qc_sample_metrics",
            Part::bytes(include_bytes!(
                "upload_files/test-data/\
                 SOD1_G93A_mouse_spinal_cord_P112_specimen_1_Multiplex_qc_sample_metrics.csv"
            ))
            .headers(csv_header.clone())
            .file_name("per_sample_outs/sample1/qc_sample_metrics.csv"),
        )
        .part(
            "summary1",
            Part::bytes(b"string_field,number_field\nstring_value,0")
                .headers(csv_header.clone())
                .file_name("qc_library_metrics.csv"),
        )
        .part(
            "summary2",
            Part::bytes(b"string_field,other_number_field\nstring_value,10")
                .headers(csv_header.clone())
                .file_name("summary.csv"),
        )
        .part(
            "metrics1",
            Part::bytes(MULTI_METRICS)
                .headers(csv_header.clone())
                .file_name("per_sample_outs/sample1/metrics_summary.csv"),
        )
        .part(
            "metrics2",
            Part::bytes(MULTI_METRICS)
                .headers(csv_header)
                .file_name("per_sample_outs/sample2/metrics_summary.csv"),
        )
        .part(
            "json_metrics",
            Part::bytes(
                br#"{"string_key": "string_value", "number": 0, "nested": {"is_this_cool": true}}"#,
            )
            .headers(
                [(CONTENT_TYPE, HeaderValue::from_static("application/json"))]
                    .into_iter()
                    .collect(),
            )
            .file_name("per_sample_outs/sample1/summary.json"),
        );

    // Thanks claude
    let request = reqwest::Client::new()
        .post("http://localhost")
        .multipart(body)
        .build()
        .unwrap();
    let mut request = Request::<reqwest::Body>::try_from(request)
        .unwrap()
        .map(Body::new);

    DefaultBodyLimit::disable().apply(&mut request);

    write_files_to_disk_and_db(
        db_pool,
        static_files_dir,
        AuthUser::dev_admin(),
        dataset_id,
        Multipart::from_request(request, &()).await.unwrap(),
    )
    .await
    .unwrap();
}

#[cfg(test)]
mod tests {
    use camino::Utf8Path;
    use tempfile::{TempDir, tempdir};

    use crate::{
        handlers::{
            chromium_datasets::{insert_test_chromium_dataset, write_test_dataset_files},
            chromium_runs::TestChromiumRunKind,
        },
        state::dev_util::{db_client_as_admin, test_db_pool},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn upload_dataset_files() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, ds) = insert_test_chromium_dataset(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let db_pool = test_db_pool();
        write_test_dataset_files(
            &db_pool,
            *ds.record.id,
            tempdir()
                .as_ref()
                .map(TempDir::path)
                .map(|p| Utf8Path::from_path(p).unwrap())
                .unwrap(),
        )
        .await;
    }
}
