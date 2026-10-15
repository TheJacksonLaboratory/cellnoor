use std::{
    collections::HashMap,
    fs::{self, File},
    str::FromStr,
};

use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
#[cfg(any(test, feature = "dev"))]
use axum::{
    body::Body,
    extract::FromRequest,
    http::{HeaderValue, Request},
};
use axum::{
    extract::{
        Multipart, Path, State,
        multipart::{Field, MultipartError},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use cellnoor_types::{
    Relation, chromium_dataset::ChromiumDatasetParsedFileData, nonempty::NonemptyString,
};
use csvranger::TenxCsvValue;
#[cfg(any(test, feature = "dev"))]
use reqwest::{
    header::CONTENT_TYPE,
    multipart::{Form, Part},
};
use strum::VariantNames;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert, Pool, Sql},
    error::error_response,
    handlers::{IdParam, specific_error_inferred_early_responses},
    state::AppState,
};

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
    static_file_dir: &Utf8Path,
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

    let chromium_dataset_dir = chromium_dataset_dir_path(static_file_dir, dataset_id);
    let project_dirs = project_dir_paths(static_file_dir, &project_names, &dataset_name);

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

        let link_paths = project_dirs.iter().map(|d| d.join(path.as_str())).collect();
        tokio::task::spawn_blocking(move || compress_and_link(&dest, link_paths))
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

fn chromium_dataset_dir_path(static_file_dir: &Utf8Path, dataset_id: Uuid) -> Utf8PathBuf {
    static_file_dir
        .join("chromium-datasets")
        .join(dataset_id.to_string())
}

fn project_dir_paths(
    static_file_dir: &Utf8Path,
    project_names: &[NonemptyString],
    dataset_name: &NonemptyString,
) -> Vec<Utf8PathBuf> {
    project_names
        .iter()
        .map(|p| {
            static_file_dir
                .join("projects")
                .join(p.as_ref())
                .join(dataset_name.as_ref())
        })
        .collect()
}

async fn write_html_to_disk(
    form_field: &mut Field<'_>,
    dest: &Utf8Path,
) -> Result<(), UploadFilesError> {
    let mut file = tokio::fs::File::create(dest).await?;

    while let Some(chunk) = form_field.chunk().await? {
        file.write_all(&chunk).await?;
    }

    file.flush().await?;

    Ok(())
}

async fn write_csv_or_json_to_disk(
    content_type: AllowedContentType,
    data: &[u8],
    dest: &Utf8Path,
) -> Result<Option<ChromiumDatasetParsedFileData>, UploadFilesError> {
    let parsed_file = parse_file(content_type, &data)?;
    tokio::fs::write(&dest, &data).await?;

    Ok(parsed_file)
}

async fn write_files_to_db(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    parsed_files: &[(UploadPath, Option<ChromiumDatasetParsedFileData>)],
) -> Result<(), DbError> {
    // Ensure the raw file is inserted first because the parsed file depends on
    // the raw file's existence
    insert_raw_files(tx, dataset_id, parsed_files).await?;
    insert_parsed_files(tx, dataset_id, parsed_files).await
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

fn compress_and_link(path: &Utf8Path, link_paths: Vec<Utf8PathBuf>) -> std::io::Result<()> {
    let compressed_path = compress_from_path(path)?;

    symlink_to_project_dirs(path, link_paths.iter().cloned())?;
    symlink_to_project_dirs(
        &compressed_path,
        link_paths
            .into_iter()
            .map(|p| p.with_added_extension("zst")),
    )
}

fn compress_from_path(original_path: &Utf8Path) -> std::io::Result<Utf8PathBuf> {
    let compressed_path = original_path.with_added_extension("zst");
    let file = File::create(&compressed_path)?;

    zstd::stream::compress_file_into(original_path, file, zstd::DEFAULT_COMPRESSION_LEVEL)?;

    Ok(compressed_path)
}

fn symlink_to_project_dirs(
    source: &Utf8Path,
    destinations: impl IntoIterator<Item = Utf8PathBuf>,
) -> std::io::Result<()> {
    for dest in destinations {
        fs::create_dir_all(dest.parent().unwrap())?;

        if dest.exists() {
            continue;
        }

        std::os::unix::fs::symlink(source, &dest).map_err(|e| {
            std::io::Error::other(format!("unable to symlink {source} to {dest}: {e}"))
        })?;
    }

    Ok(())
}

fn parse_file(
    content_type: AllowedContentType,
    data: &[u8],
) -> Result<Option<ChromiumDatasetParsedFileData>, UploadFilesError> {
    match content_type {
        AllowedContentType::Csv => parse_csv(data).map(Some),
        AllowedContentType::Json => serde_json::from_reader(data)
            .map(|data| ChromiumDatasetParsedFileData::Json { data })
            .map(Some)
            .map_err(|e| UploadFilesError::Parse {
                message: format!("failed to parse JSON: {e}"),
            }),
        AllowedContentType::Html => Ok(None),
    }
}

async fn insert_raw_files(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    paths: &[(UploadPath, Option<ChromiumDatasetParsedFileData>)],
) -> Result<(), DbError> {
    let files: Vec<_> = paths
        .iter()
        .map(|(p, _)| NewRawFile {
            dataset_id,
            path: p.as_str(),
        })
        .collect();

    tx.insert_many(&files).await?;

    Ok(())
}

struct NewRawFile<'a> {
    dataset_id: Uuid,
    path: &'a str,
}

impl Relation for NewRawFile<'_> {
    const NAME: &'static str = "chromium_dataset_raw_file";
}

impl Insert for NewRawFile<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self { dataset_id, path } = self;

        vec![("dataset_id", dataset_id), ("path", path)]
    }
}

async fn insert_parsed_files(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    parsed_files: &[(UploadPath, Option<ChromiumDatasetParsedFileData>)],
) -> Result<(), DbError> {
    let records: Vec<_> = parsed_files
        .iter()
        .filter_map(|(path, data)| {
            data.as_ref().map(|d| NewParsedFile {
                dataset_id,
                path: path.as_str(),
                data: postgres_types::Json(d),
            })
        })
        .collect();

    tx.insert_many(&records).await?;

    Ok(())
}

struct NewParsedFile<'a> {
    dataset_id: Uuid,
    path: &'a str,
    data: postgres_types::Json<&'a ChromiumDatasetParsedFileData>,
}

impl Relation for NewParsedFile<'_> {
    const NAME: &'static str = "chromium_dataset_parsed_file";
}

impl Insert for NewParsedFile<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            dataset_id,
            path,
            data,
        } = self;

        vec![("dataset_id", dataset_id), ("path", path), ("data", data)]
    }
}

fn parse_csv(data: &[u8]) -> Result<ChromiumDatasetParsedFileData, UploadFilesError> {
    type ParsedData = Vec<HashMap<String, TenxCsvValue>>;

    let mut reader = csv::Reader::from_reader(data);

    let mut parsed = reader
        .deserialize()
        .collect::<Result<ParsedData, _>>()
        .map_err(|e| UploadFilesError::Parse {
            message: format!("failed to parse CSV: {e}"),
        })?;

    let parsed = if parsed.len() == 1 {
        ChromiumDatasetParsedFileData::SingleRowCsv {
            data: parsed.swap_remove(0),
        }
    } else {
        ChromiumDatasetParsedFileData::MultiRowCsv { data: parsed }
    };

    Ok(parsed)
}

#[derive(
    Debug, Clone, Copy, strum::EnumString, strum::AsRefStr, PartialEq, strum::VariantNames,
)]
enum AllowedContentType {
    #[strum(serialize = "text/csv")]
    Csv,
    #[strum(serialize = "text/html")]
    Html,
    #[strum(serialize = "application/json")]
    Json,
}

impl AllowedContentType {
    fn from_multipart_form_field(field: &Field<'_>) -> Result<Self, UploadFilesError> {
        field
            .content_type()
            .map(AllowedContentType::from_str)
            .ok_or(UploadFilesError::ContentType {
                message: "file-upload must have content type".to_owned(),
            })?
            .map_err(|_| UploadFilesError::ContentType {
                message: format!("content-type must be one of: {:?}", Self::VARIANTS),
            })
    }
}

#[derive(Debug)]
struct UploadPath(Utf8PathBuf);

impl UploadPath {
    fn new(filename: Option<&str>) -> Result<Self, UploadFilesError> {
        const ALLOWED_FILENAMES: [&str; 7] = [
            "metrics_summary.csv",
            "qc_library_metrics.csv",
            "qc_report.html",
            "qc_sample_metrics.csv",
            "summary.csv",
            "summary.json",
            "web_summary.html",
        ];

        let filename_error = Err(UploadFilesError::Filename {
            message: format!(
                "uploaded files must have a filename which is one of {:?}",
                ALLOWED_FILENAMES
            ),
        });

        let Some(path) = filename.map(Utf8Path::new) else {
            return filename_error;
        };

        if path
            .file_name()
            .is_none_or(|f| !ALLOWED_FILENAMES.contains(&f))
        {
            return filename_error;
        }

        let components: Vec<_> = path.components().collect();

        if !components
            .iter()
            .all(|c| matches!(c, Utf8Component::Normal(_)))
        {
            return Err(UploadFilesError::Filename {
                message: "path must be relative and cannot contain '.' or '..'".to_owned(),
            });
        }

        match components.as_slice() {
            [_] | [Utf8Component::Normal("per_sample_outs"), _, _] => Ok(Self(path.to_owned())),
            _ => Err(UploadFilesError::Filename {
                message: "files nested into a directory must be nested into a \
                          'per_sample_outs/sample_name/' directory"
                    .to_owned(),
            }),
        }
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(any(test, feature = "dev"))]
pub async fn write_test_dataset_files(dataset_id: Uuid) {
    use tempfile::tempdir;

    use crate::state::dev_util::test_db_pool;

    let tmp = tempdir().unwrap();
    let static_file_dir = option_env!("CELLNOOR_API__STATIC_FILE_DIR")
        .map(Utf8Path::new)
        .unwrap_or(Utf8Path::from_path(tmp.path()).unwrap());

    let body = Form::new()
        .part(
            "qc_report.html",
            Part::bytes(b"QC report")
                .headers(
                    [(CONTENT_TYPE, HeaderValue::from_static("text/html"))]
                        .into_iter()
                        .collect(),
                )
                .file_name("qc_report.html"),
        )
        .part("metrics1", Part::bytes(b"Category,Library Type,Grouped By,Group Name,Metric Name,Metric Value\nCells,Gene Expression,,,Cells,1000\nCells,Gene Expression,,,Confidently mapped antisense,0.0")
        .headers(
            [(CONTENT_TYPE, HeaderValue::from_static("text/csv"))]
                .into_iter()
                .collect(),
        )
        .file_name("per_sample_outs/sample1/metrics_summary.csv"))
        .part(
            "metrics2",
            Part::bytes(b"Category,Library Type,Grouped By,Group Name,Metric Name,Metric Value\nCells,Gene Expression,,,Cells,1000\nCells,Gene Expression,,,Confidently mapped antisense,0.0")
            .headers(
                [(CONTENT_TYPE, HeaderValue::from_static("text/csv"))]
                    .into_iter()
                    .collect(),
            )
            .file_name("per_sample_outs/sample2/metrics_summary.csv"),
        );

    // Thanks claude
    let request = reqwest::Client::new()
        .post("http://localhost")
        .multipart(body)
        .build()
        .unwrap();
    let request = Request::<reqwest::Body>::try_from(request)
        .unwrap()
        .map(Body::new);

    write_files_to_disk_and_db(
        &test_db_pool(),
        static_file_dir,
        AuthUser::dev_admin(),
        dataset_id,
        Multipart::from_request(request, &()).await.unwrap(),
    )
    .await
    .unwrap();
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::UploadPath;

    #[test]
    fn empty_filename() {
        UploadPath::new(Some("")).unwrap_err();
    }

    #[test]
    fn root_filename() {
        UploadPath::new(Some("/file")).unwrap_err();
    }

    #[test]
    fn correct_filenames() {
        let path = UploadPath::new(Some("metrics_summary.csv")).unwrap();

        assert_eq!(path.as_str(), "metrics_summary.csv");

        let path =
            UploadPath::new(Some("per_sample_outs/sample_name/metrics_summary.csv")).unwrap();

        assert_eq!(
            path.as_str(),
            "per_sample_outs/sample_name/metrics_summary.csv"
        );
    }

    #[test]
    fn non_normal_component() {
        UploadPath::new(Some("a/../web_summary.html")).unwrap_err();
    }
}
