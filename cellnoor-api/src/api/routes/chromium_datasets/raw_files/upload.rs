use std::{collections::HashMap, str::FromStr};

use axum::extract::{Multipart, Path, State, multipart::Field};
use axum_extra::TypedHeader;
use camino::Utf8Path;
use cellnoor_models::{
    IdParameter,
    chromium_dataset::metrics::{
        ParsedMetricsData,
        multi_row_csv::{self},
    },
};
use cellnoor_schema::chromium_dataset_raw_files;
use csvranger::TenxCsvValue;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use headers::ContentEncoding;
use uuid::Uuid;

use crate::{
    api::routes::chromium_datasets::{ParsedChromiumDatasetFile, raw_files::MAX_N_SAMPLES},
    db::{self},
    state::AppState,
};

pub async fn upload_files(
    state: State<AppState>,
    Path(IdParameter { id }): Path<IdParameter>,
    TypedHeader(encoding): TypedHeader<ContentEncoding>,
    mut multipart_form: Multipart,
) -> Result<(), db::Error> {
    // Every sample has a web_summary.html and a metrics_summary.csv (only the
    // latter of which is parsed), and cellranger outputs some dataset-level files
    // as well
    let mut extracted_raw_files = Vec::with_capacity((MAX_N_SAMPLES * 2) + 4);
    let mut extracted_parsed_files = Vec::with_capacity(MAX_N_SAMPLES + 4);

    while let Some(field) = multipart_form
        .next_field()
        .await
        .map_err(|e| db::DataError::new_other(&e.body_text()))?
    {
        let content_type = AllowedContentType::from_multipart_form_field(&field)?;
        let path = extract_path(field.file_name()).map(str::to_owned)?;
        let raw_content: Vec<_> = field
            .bytes()
            .await
            .map_err(|e| db::DataError::Other {
                message: format!("failed to read data from multipart/form-data: {e}"),
            })?
            .into();

        let (parsed_file, raw_file) = match content_type {
            AllowedContentType::Csv => (
                ParsedChromiumDatasetFile::from_csv(id, path.clone(), &raw_content).map(Some)?,
                NewRawFile::new_uncompressed(id, path, content_type, raw_content),
            ),
            AllowedContentType::Html => {
                if !encoding.contains("zstd") {
                    return Err(db::DataError::new_other(
                        "content-encoding must be 'zstd' for HTML files",
                    ))?;
                }

                (
                    None,
                    NewRawFile::new_html(id, path, Some("zstd"), raw_content),
                )
            }
            AllowedContentType::Json => (
                ParsedChromiumDatasetFile::from_json(id, path.clone(), &raw_content).map(Some)?,
                NewRawFile::new_uncompressed(id, path, content_type, raw_content),
            ),
        };

        extracted_raw_files.push(raw_file);
        if let Some(parsed_file) = parsed_file {
            extracted_parsed_files.push(parsed_file);
        }
    }

    // It's important that we grab the database connection HERE rather than as an
    // axum extractor because this allows all the data to be read from the HTTP
    // stream before we actually need a db connection, preventing a deadlock (I
    // actually experienced this)
    let db_conn = state.db_conn().await?;

    tokio::try_join!(
        insert_raw_files(&extracted_raw_files, &db_conn),
        insert_parsed_files(&extracted_parsed_files, &db_conn)
    )?;

    Ok(())
}

async fn insert_raw_files(
    files: &[NewRawFile],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    use cellnoor_schema::chromium_dataset_raw_files::dsl::*;

    let n_files_inserted = diesel::insert_into(chromium_dataset_raw_files)
        .values(files)
        .on_conflict((dataset_id, path))
        .do_nothing()
        .execute(&mut db_conn)
        .await?;

    if n_files_inserted != files.len() {
        let mut upserts = Vec::with_capacity(files.len());
        for f in files {
            upserts.push(upsert_raw_file(f, db_conn));
        }

        futures::future::try_join_all(upserts).await?;
    }

    Ok(())
}

async fn upsert_raw_file(
    file: &NewRawFile,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    use cellnoor_schema::chromium_dataset_raw_files::dsl::*;

    diesel::insert_into(chromium_dataset_raw_files)
        .values(file)
        .on_conflict((dataset_id, path))
        .do_update()
        .set(file)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn insert_parsed_files(
    files: &[ParsedChromiumDatasetFile],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    use cellnoor_schema::chromium_dataset_parsed_files::dsl::*;

    let n_files_inserted = diesel::insert_into(chromium_dataset_parsed_files)
        .values(files)
        .on_conflict((dataset_id, path))
        .do_nothing()
        .execute(&mut db_conn)
        .await?;

    if n_files_inserted != files.len() {
        let mut upserts = Vec::with_capacity(files.len());
        for f in files {
            upserts.push(upsert_parsed_file(f, db_conn));
        }

        futures::future::try_join_all(upserts).await?;
    }

    Ok(())
}

async fn upsert_parsed_file(
    file: &ParsedChromiumDatasetFile,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    use cellnoor_schema::chromium_dataset_parsed_files::dsl::*;

    diesel::insert_into(chromium_dataset_parsed_files)
        .values(file)
        .on_conflict((dataset_id, path))
        .do_update()
        .set(file)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

#[derive(Insertable, AsChangeset, Identifiable)]
#[diesel(table_name = chromium_dataset_raw_files, check_for_backend(Pg), primary_key(dataset_id, path))]
struct NewRawFile {
    dataset_id: Uuid,
    path: String,
    content_type: &'static str,
    raw_content: Vec<u8>,
    content_encoding: Option<&'static str>,
}

impl NewRawFile {
    fn new_uncompressed(
        dataset_id: Uuid,
        path: String,
        content_type: AllowedContentType,
        raw_content: Vec<u8>,
    ) -> Self {
        Self {
            dataset_id,
            path,
            content_type: content_type.into(),
            raw_content,
            content_encoding: None,
        }
    }

    fn new_html(
        dataset_id: Uuid,
        path: String,
        content_encoding: Option<&'static str>,
        raw_content: Vec<u8>,
    ) -> Self {
        Self {
            dataset_id,
            path,
            content_type: AllowedContentType::Html.into(),
            raw_content,
            content_encoding,
        }
    }
}

impl ParsedChromiumDatasetFile {
    fn from_csv(dataset_id: Uuid, path: String, raw_content: &[u8]) -> Result<Self, db::DataError> {
        parse_ranger_csv(raw_content)
            .map(ParsedMetricsData::KeyValue)
            .or_else(|_| parse_cellrangermulti_csv(raw_content).map(ParsedMetricsData::Tabular))
            .map(|data| Self {
                dataset_id,
                path,
                data,
            })
    }

    fn from_json(
        dataset_id: Uuid,
        path: String,
        raw_content: &[u8],
    ) -> Result<Self, db::DataError> {
        serde_json::from_slice(raw_content)
            .map_err(|e| db::DataError::new_other(&format!("failed to parse JSON: {e}")))
            .map(ParsedMetricsData::KeyValue)
            .map(|data| Self {
                dataset_id,
                path,
                data,
            })
    }
}

fn parse_ranger_csv(
    raw_content: &[u8],
) -> Result<Vec<HashMap<String, TenxCsvValue>>, db::DataError> {
    let mut csv = csv::Reader::from_reader(raw_content);

    csv.deserialize()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| db::DataError::Other {
            message: e.to_string(),
        })

    // let header: Vec<_> = csv
    //     .headers()
    //     .map_err(|e| db::DataError::new_other(&format!("failed to parse CSV headers: {e}")))?
    //     .into_iter()
    //     .map(str::to_owned)
    //     .collect();
    // let header_len = header.len();
    // let mut records = csv.records();

    // let mut parsed_data = HashMap::with_capacity(header_len);

    // // Manual insertion into the map is preferred over `collect` because the latter
    // // would require an extra iteration to transform `Vec<Result<_>>` to
    // // `Result<Vec<_>>` before constructing the two-tuple
    // for (field_name, field_value) in header.into_iter().zip(first_record.iter()) {
    //     // Some of the fields of these CSVs have strings instead of numbers. If that's
    //     // the case, then we just insert the original string
    //     parsed_data.insert(
    //         field_name,
    //         parse_str_as_number(field_value)
    //             .map_or_else(|_| Value::String(field_value.to_owned()), Value::Number),
    //     );
    // }

    // Ok(parsed_data)
}

fn parse_cellrangermulti_csv(raw_content: &[u8]) -> Result<Vec<multi_row_csv::Row>, db::DataError> {
    let mut csv = csv::Reader::from_reader(raw_content);

    let map_err = |e| db::DataError::new_other(&format!("failed to parse multi-row CSV: {e}"));

    let headers = csv.headers().map_err(map_err)?.clone();

    let mut parsed_data = Vec::with_capacity(100);
    for record in csv.records() {
        let record = record.map_err(map_err)?;

        let simple_fields: multi_row_csv::SimpleFields =
            record.deserialize(Some(&headers)).map_err(map_err)?;

        let metric_value_str = record.get(5).ok_or(db::DataError::new_other(
            "failed to parse multi-row CSV: column 'Metric Value' is missing",
        ))?;

        let extracted_metric_value = match metric_value_str.split_once(' ') {
            Some((actual_value, _)) => actual_value,
            None => metric_value_str,
        };

        parsed_data.push(multi_row_csv::Row::new(
            simple_fields,
            TenxCsvValue::from_legacy_csv_value(extracted_metric_value),
        ));
    }

    Ok(parsed_data)
}

#[derive(Debug, Clone, Copy, strum::EnumString, strum::IntoStaticStr, PartialEq)]
enum AllowedContentType {
    #[strum(serialize = "text/csv")]
    Csv,
    #[strum(serialize = "text/html")]
    Html,
    #[strum(serialize = "application/json")]
    Json,
}

impl AllowedContentType {
    fn from_multipart_form_field(field: &Field<'_>) -> Result<Self, db::DataError> {
        field
            .content_type()
            .map(AllowedContentType::from_str)
            .ok_or(db::DataError::new_other(
                "file-upload must have content type",
            ))?
            .map_err(|e| db::DataError::new_other(&format!("failed to parse content-type: {e}")))
    }
}

fn extract_path(filename: Option<&str>) -> Result<&str, db::DataError> {
    const ALLOWED_FILENAMES: [&str; 7] = [
        "metrics_summary.csv",
        "qc_library_metrics.csv",
        "qc_report.html",
        "qc_sample_metrics.csv",
        "summary.csv",
        "summary.json",
        "web_summary.html",
    ];

    let Some(path) = filename.map(Utf8Path::new) else {
        return Err(db::DataError::new_other("uploaded file must have filename"));
    };

    if path
        .file_name()
        .is_none_or(|f| !ALLOWED_FILENAMES.contains(&f))
    {
        return Err(db::DataError::new_other(&format!(
            "invalid filename: {path}"
        )));
    }

    if path.is_absolute() {
        return Err(db::DataError::new_other(
            "file cannot be in the root directory",
        ));
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

    let per_sample_outs_error = Err(db::DataError::new_other(
        "files nested into a directory must be nested into a 'per_sample_outs/sample_name/' \
         directory",
    ));

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
    use rstest::rstest;

    use super::{extract_path, parse_cellrangermulti_csv, parse_ranger_csv};

    #[rstest]
    fn empty_filename() {
        assert!(extract_path(Some("")).is_err());
    }

    #[rstest]
    fn root_filename() {
        assert!(extract_path(Some("/file")).is_err());
    }

    #[rstest]
    fn correct_filenames() {
        let path = extract_path(Some("metrics_summary.csv")).unwrap();

        assert_eq!(path, "metrics_summary.csv");

        let path = extract_path(Some("per_sample_outs/sample_name/metrics_summary.csv")).unwrap();

        assert_eq!(path, "per_sample_outs/sample_name/metrics_summary.csv");
    }

    #[rstest]
    fn cellranger_count() {
        let raw_content = include_bytes!("test-data/cellranger_count.csv");
        let parsed_data = parse_ranger_csv(raw_content).unwrap();
        let parsed_data = &parsed_data[0];

        assert_eq!(
            parsed_data["Estimated Number of Cells"].as_i64().unwrap(),
            65_558
        );

        assert!(0.378 - parsed_data["Sequencing Saturation"].as_f64().unwrap() < 0.01);
    }

    #[rstest]
    fn cellranger_arc_count() {
        let raw_content = include_bytes!("test-data/cellranger-arc_count.csv");
        let parsed_data = parse_ranger_csv(raw_content).unwrap();
        let parsed_data = &parsed_data[0];

        assert_eq!(
            parsed_data["Estimated number of cells"].as_i64().unwrap(),
            11_673
        );

        assert_eq!(parsed_data["Sample ID"].as_str().unwrap(), "Sample0");

        assert_eq!(
            parsed_data["ATAC Confidently mapped read pairs"]
                .as_f64()
                .unwrap(),
            0.8937
        );
    }

    #[rstest]
    fn cellranger_multi() {
        let raw_content = include_bytes!("test-data/cellranger_multi.csv");
        let parsed_data = parse_cellrangermulti_csv(raw_content).unwrap();

        let row = &parsed_data[0];
        assert_eq!(row.metric_value.as_i64().unwrap(), 1866);

        let row = &parsed_data[1];
        assert_eq!(row.metric_value.as_f64().unwrap(), 0.9314);

        let row = &parsed_data[13];
        assert_eq!(
            row.simple_fields.metric_name,
            "Cells detected in this sample"
        );

        assert_eq!(row.metric_value.as_i64().unwrap(), 1866);
    }

    #[test]
    fn cellranger_multi10() {
        let raw_content = include_bytes!("test-data/cellranger_multi.10.qc_sample_metrics.csv");
        let parsed_data = parse_ranger_csv(raw_content).unwrap();

        let row = &parsed_data[0];
        assert_eq!(row["GEX: Cells"].as_i64().unwrap(), 16410);
    }
}
