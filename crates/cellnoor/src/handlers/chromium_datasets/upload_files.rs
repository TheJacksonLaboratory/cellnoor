use std::{collections::HashMap, str::FromStr};

use axum::extract::{Multipart, Path, State, multipart::Field};
use bytes::Bytes;
use camino::Utf8Path;
use csvranger::TenxCsvValue;
use nonempty::NonemptyString;
use strum::VariantNames;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn upload_files(
    State(app_state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: dataset_id }): Path<IdParam>,
    mut files: Multipart,
) -> Result<(), Error> {
    // Technically could be up to 768 but that's not happening any time soon
    const N_FILES: usize = 128;

    let mut raw_files = Vec::with_capacity(N_FILES);
    let mut parsed_files = Vec::with_capacity(N_FILES);

    while let Some(field) = files
        .next_field()
        .await
        .map_err(|e| ErrorInner::FileUpload {
            message: e.to_string(),
        })?
    {
        let ProcessedFile {
            path,
            raw_file,
            parsed_file,
        } = process_file(field).await?;

        raw_files.push((path.clone(), raw_file));
        parsed_files.push((path, parsed_file));
    }

    // Grab the db client as late as possible and get rid of it as early as possible
    // by putting it in a scoped block. This is performed first to utilize the
    // db's permissions checks
    {
        let mut client = app_state.db_client(user).await?;
        let tx = client.begin().await?;

        let db_file_insertions = parsed_files
            .iter()
            .map(|(path, file)| write_file_to_db(&tx, dataset_id, path, file.as_ref()));
        futures::future::try_join_all(db_file_insertions).await?;

        tx.commit().await?;
    }

    tokio::task::spawn_blocking(move || {
        write_fileset_to_disk(app_state.static_files_dir(), dataset_id, &raw_files)
    });

    Ok(())
}

async fn write_file_to_db(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    path: &NonemptyString,
    parsed_file: Option<&serde_json::Value>,
) -> Result<(), ErrorInner> {
    // Ensure the raw file is inserted first because the parsed file depends on the
    // raw file's existence
    insert_raw_file(tx, dataset_id, path).await?;
    insert_parsed_file(tx, dataset_id, path, parsed_file).await
}

fn write_fileset_to_disk(
    static_file_dir: &Utf8Path,
    dataset_id: Uuid,
    raw_files: &[(NonemptyString, Bytes)],
) -> Result<(), ErrorInner> {
    for (path, raw_file) in raw_files {
        write_file_to_disk(static_file_dir, dataset_id, path, None, raw_file)?;

        let compressed_file = compress_file(raw_file)?;
        write_file_to_disk(
            static_file_dir,
            dataset_id,
            path,
            Some("zst"),
            &compressed_file,
        )?;
    }

    Ok(())
}

fn write_file_to_disk(
    static_file_dir: &Utf8Path,
    dataset_id: Uuid,
    path: &NonemptyString,
    extension: Option<&str>,
    raw_file: &[u8],
) -> Result<(), ErrorInner> {
    let mut path = static_file_dir
        .join("chromium-datasets")
        .join(dataset_id.to_string())
        .join(path.as_ref());

    if let Some(ext) = extension {
        path.add_extension(ext);
    }

    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    std::fs::write(&path, &raw_file).map_err(|e| ErrorInner::Other {
        message: format!("failed to write file: {e}"),
        sql_state: None,
    })?;

    Ok(())
}

fn compress_file(raw_file: &[u8]) -> Result<Vec<u8>, ErrorInner> {
    let compression_level = if cfg!(debug_assertions) {
        zstd::DEFAULT_COMPRESSION_LEVEL
    } else {
        19
    };

    zstd::encode_all(raw_file, compression_level).map_err(|e| ErrorInner::FileUpload {
        message: format!("failed to write compressed file to disk: {e}"),
    })
}

async fn process_file(form_field: Field<'_>) -> Result<ProcessedFile, ErrorInner> {
    let content_type = AllowedContentType::from_multipart_form_field(&form_field)?;

    let path = extract_path(form_field.file_name())
        .map(str::to_owned)
        .map(NonemptyString::new)?
        .unwrap();

    let raw_data = form_field
        .bytes()
        .await
        .map_err(|e| ErrorInner::FileUpload {
            message: format!("failed to extract data from form field: {e}"),
        })?;

    Ok(ProcessedFile {
        path,
        parsed_file: parse_file(content_type, &raw_data)?,
        raw_file: raw_data,
    })
}

struct ProcessedFile {
    path: NonemptyString,
    raw_file: Bytes,
    parsed_file: Option<serde_json::Value>,
}

fn parse_file(
    content_type: AllowedContentType,
    data: &[u8],
) -> Result<Option<serde_json::Value>, ErrorInner> {
    match content_type {
        AllowedContentType::Csv => parse_csv(data).map(Some),
        AllowedContentType::Json => {
            serde_json::from_slice(data)
                .map(Some)
                .map_err(|e| ErrorInner::FileUpload {
                    message: format!("failed to parse JSON: {e}"),
                })
        }
        AllowedContentType::Html => Ok(None),
    }
}

async fn insert_raw_file(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    path: &NonemptyString,
) -> Result<(), ErrorInner> {
    db::insert_into_no_returning(
        tx,
        "chromium_dataset_raw_file",
        &NewRawFile { dataset_id, path },
    )
    .await?;

    Ok(())
}

struct NewRawFile<'a> {
    dataset_id: Uuid,
    path: &'a NonemptyString,
}

impl AsFieldValuePairs<&'static str, 2> for NewRawFile<'_> {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, &'static str, 2> {
        let Self { dataset_id, path } = self;

        [("dataset_id", dataset_id), ("path", path)]
    }
}

async fn insert_parsed_file(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    path: &NonemptyString,
    parsed_file: Option<&serde_json::Value>,
) -> Result<(), ErrorInner> {
    let Some(parsed_file) = parsed_file else {
        return Ok(());
    };

    let record = NewParsedFile {
        dataset_id,
        path,
        data: parsed_file,
    };

    db::insert_into_no_returning(tx, "chromium_dataset_parsed_file", &record).await?;

    Ok(())
}

struct NewParsedFile<'a> {
    dataset_id: Uuid,
    path: &'a NonemptyString,
    data: &'a serde_json::Value,
}

impl AsFieldValuePairs<&'static str, 3> for NewParsedFile<'_> {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, &'static str, 3> {
        let Self {
            dataset_id,
            path,
            data,
        } = self;

        [("dataset_id", dataset_id), ("path", path), ("data", data)]
    }
}

fn parse_csv(data: &[u8]) -> Result<serde_json::Value, ErrorInner> {
    type ParsedData = Vec<HashMap<String, TenxCsvValue>>;

    fn csv_value_to_serde(val: TenxCsvValue) -> serde_json::Value {
        match val {
            TenxCsvValue::F64(f) => {
                serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap())
            }
            TenxCsvValue::I64(i) => serde_json::Value::Number(i.into()),
            TenxCsvValue::String(s) => serde_json::Value::String(s),
        }
    }

    let mut reader = csv::Reader::from_reader(data);

    let parsed = reader
        .deserialize()
        .collect::<Result<ParsedData, _>>()
        .map_err(|e| ErrorInner::FileUpload {
            message: format!("failed to parse CSV: {e}"),
        })?;

    Ok(parsed
        .into_iter()
        .map(|obj| {
            obj.into_iter()
                .map(|(k, v)| (k, csv_value_to_serde(v)))
                .collect::<serde_json::Value>()
        })
        .collect())
}

#[derive(
    Debug, Clone, Copy, strum::EnumString, strum::IntoStaticStr, PartialEq, strum::VariantNames,
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
    fn from_multipart_form_field(field: &Field<'_>) -> Result<Self, ErrorInner> {
        field
            .content_type()
            .map(AllowedContentType::from_str)
            .ok_or(ErrorInner::FileUpload {
                message: "file-upload must have content type".to_owned(),
            })?
            .map_err(|_| ErrorInner::FileUpload {
                message: format!("content-type must be one of: {:?}", Self::VARIANTS),
            })
    }
}

fn extract_path(filename: Option<&str>) -> Result<&str, ErrorInner> {
    const ALLOWED_FILENAMES: [&str; 7] = [
        "metrics_summary.csv",
        "qc_library_metrics.csv",
        "qc_report.html",
        "qc_sample_metrics.csv",
        "summary.csv",
        "summary.json",
        "web_summary.html",
    ];

    let filename_error = Err(ErrorInner::FileUpload {
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

    if path.is_absolute() {
        return Err(ErrorInner::FileUpload {
            message: "path cannot be absolute".to_owned(),
        });
    }

    let path_as_str = path.as_str();

    let parent = match path.parent() {
        None => {
            return Ok(path_as_str);
        }
        Some(p) if p.file_name().is_none() => {
            return Ok(path_as_str);
        }
        Some(p) => p,
    };

    let per_sample_outs_error = Err(ErrorInner::FileUpload {
        message: "files nested into a directory must be nested into a \
                  'per_sample_outs/sample_name/' directory"
            .to_owned(),
    });

    let Some(parent) = parent.parent() else {
        return per_sample_outs_error;
    };

    if parent != "per_sample_outs" {
        return per_sample_outs_error;
    }

    Ok(path_as_str)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::extract_path;

    #[test]
    fn empty_filename() {
        extract_path(Some("")).unwrap_err();
    }

    #[test]
    fn root_filename() {
        extract_path(Some("/file")).unwrap_err();
    }

    #[test]
    fn correct_filenames() {
        let path = extract_path(Some("metrics_summary.csv")).unwrap();

        assert_eq!(path, "metrics_summary.csv");

        let path = extract_path(Some("per_sample_outs/sample_name/metrics_summary.csv")).unwrap();

        assert_eq!(path, "per_sample_outs/sample_name/metrics_summary.csv");
    }
}
