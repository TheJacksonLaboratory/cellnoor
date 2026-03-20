use std::{collections::HashMap, str::FromStr};

use axum::extract::{Multipart, Path, State, multipart::Field};
use camino::Utf8Path;
use cellnoor_models::{
    IdParameter,
    chromium_dataset::metrics::{
        ParsedMetricsData,
        multi_row_csv::{self},
    },
};
use cellnoor_schema::chromium_dataset_files;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use heck::ToSnekCase;
use serde_json::{Number, Value};
use uuid::Uuid;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn upload_file(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    mut multipart_form: Multipart,
) -> Result<(), db::Error> {
    let mut extracted_files = Vec::with_capacity(32);

    while let Some(field) = multipart_form
        .next_field()
        .await
        .map_err(|e| db::DataError::new_other(&e.body_text()))?
    {
        let new_file = NewFile::from_multipart_form_field(id, field).await?;
        extracted_files.push(new_file);
    }

    insert_files(&extracted_files, &db_conn).await?;

    Ok(())
}

async fn insert_files(
    files: &[NewFile],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    use cellnoor_schema::chromium_dataset_files::dsl::*;

    diesel::insert_into(chromium_dataset_files)
        .values(files)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

#[derive(Insertable)]
#[diesel(table_name = chromium_dataset_files, check_for_backend(Pg))]
struct NewFile {
    dataset_id: Uuid,
    path: String,
    content_type: &'static str,
    raw_content: Vec<u8>,
    parsed_data: Option<ParsedMetricsData>,
}

impl NewFile {
    async fn from_multipart_form_field(
        dataset_id: Uuid,
        field: Field<'_>,
    ) -> Result<Self, db::DataError> {
        let content_type = AllowedContentType::from_multipart_form_field(&field)?;
        let path = extract_path(field.file_name()).map(str::to_owned)?;
        let raw_content = field
            .bytes()
            .await
            .map_err(|e| db::DataError::new_other(&e.body_text()))?;

        let parsed_data = match content_type {
            AllowedContentType::Csv => parse_single_row_csv(&raw_content)
                .map(ParsedMetricsData::KeyValue)
                .or_else(|_| parse_multi_row_csv(&raw_content).map(ParsedMetricsData::Tabular))
                .map(Some)?,
            AllowedContentType::Html => None,
            AllowedContentType::Json => serde_json::from_slice(&raw_content)
                .map_err(|e| db::DataError::new_other(&e.to_string()))?,
        };

        Ok(Self {
            dataset_id,
            path,
            content_type: content_type.into(),
            // It should be possible to just get the underlying slice as `&[u8]` but it causes
            // lifetime issues and I don't feel like dealing with those
            raw_content: raw_content.to_vec(),
            parsed_data,
        })
    }
}

fn parse_single_row_csv(raw_content: &[u8]) -> Result<HashMap<String, Value>, db::DataError> {
    let mut csv = csv::Reader::from_reader(raw_content);

    let header = csv
        .headers()
        .map_err(|e| db::DataError::new_other(&format!("failed to parse CSV headers: {e}")))?;
    let header_len = header.len();
    let snake_case_header: Vec<String> = header.iter().map(snake_case_field_name).collect();
    let mut records = csv.records();

    let n_rows_err = Err(db::DataError::new_other("expected exactly one row in CSV"));

    let first_record = match records.next() {
        Some(rec) => {
            rec.map_err(|e| db::DataError::new_other(&format!("failed to parse CSV row: {e}")))?
        }
        None => {
            return n_rows_err;
        }
    };

    if records.next().is_some() {
        return n_rows_err;
    }

    let mut parsed_data = HashMap::with_capacity(header_len);

    // Manual insertion into the map is preferred over `collect` because the latter
    // would require an extra iteration to transform `Vec<Result<_>>` to
    // `Result<Vec<_>>` before constructing the two-tuple
    for (field_name, field_value) in snake_case_header.into_iter().zip(first_record.iter()) {
        // Some of the fields of these CSVs have strings instead of numbers. If that's
        // the case, then we just insert the original string
        parsed_data.insert(
            field_name,
            parse_str_as_number(field_value)
                .map_or_else(|_| Value::String(field_value.to_owned()), Value::Number),
        );
    }

    Ok(parsed_data)
}

fn parse_multi_row_csv(raw_content: &[u8]) -> Result<Vec<multi_row_csv::Row>, db::DataError> {
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

        let metric_value = parse_str_as_number(extracted_metric_value).map_or_else(
            |_| Value::String(metric_value_str.to_owned()),
            Value::Number,
        );

        parsed_data.push(multi_row_csv::Row::new(simple_fields, metric_value));
    }

    Ok(parsed_data)
}

fn snake_case_field_name(field_name: &str) -> String {
    let field_name = field_name.replace("UMIs", "umis");
    field_name.to_snek_case()
}

fn parse_str_as_number(value: &str) -> Result<Number, <Number as FromStr>::Err> {
    if let Ok(value) = value.parse() {
        return Ok(value);
    }

    let original_str_value = value;
    let value_without_shit = value.replace([',', '%', '"'], "");

    let mut value_as_number = Number::from_str(&value_without_shit)?;
    if original_str_value.contains('%') {
        value_as_number =
            Number::from_f64(value_as_number.as_f64().map(|f| f / 100.0).unwrap()).unwrap();
    }

    Ok(value_as_number)
}

#[derive(Debug, Clone, Copy, strum::EnumString, strum::IntoStaticStr)]
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
            .map_err(|e| db::DataError::new_other(&e.to_string()))
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
        return Err(db::DataError::new_other("invalid filename"));
    }

    if path.is_absolute() {
        return Err(db::DataError::new_other(
            "file cannot be in the root directory",
        ));
    }

    let Some(parent) = path.parent() else {
        return Ok(path.as_str());
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

    Ok(path.as_str())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{extract_path, parse_multi_row_csv, parse_single_row_csv};

    #[rstest]
    fn empty_filename() {
        assert!(extract_path(Some("")).is_err());
    }

    #[rstest]
    fn root_filename() {
        assert!(extract_path(Some("/file")).is_err());
    }

    #[rstest]
    fn correct_filename() {
        let path = extract_path(Some("parent/file")).unwrap();

        assert_eq!(path, "parent/file");

        let path = extract_path(Some("grandparent/parent/file")).unwrap();

        assert_eq!(path, "grandparent/parent/file");
    }

    #[rstest]
    fn cellranger_count() {
        let raw_content = include_bytes!("test-data/cellranger_count.csv");
        let parsed_data = parse_single_row_csv(raw_content).unwrap();

        assert_eq!(
            parsed_data["estimated_number_of_cells"].as_i64().unwrap(),
            65_558
        );

        assert!(0.378 - parsed_data["sequencing_saturation"].as_f64().unwrap() < 0.01);
    }

    #[rstest]
    fn cellranger_arc_count() {
        let raw_content = include_bytes!("test-data/cellranger-arc_count.csv");
        let parsed_data = parse_single_row_csv(raw_content).unwrap();

        assert_eq!(
            parsed_data["estimated_number_of_cells"].as_i64().unwrap(),
            11_673
        );

        assert_eq!(parsed_data["sample_id"].as_str().unwrap(), "Sample0");

        assert_eq!(
            parsed_data["atac_confidently_mapped_read_pairs"]
                .as_f64()
                .unwrap(),
            0.8937
        );
    }

    #[rstest]
    fn cellranger_multi() {
        let raw_content = include_bytes!("test-data/cellranger_multi.csv");
        let parsed_data = parse_multi_row_csv(raw_content).unwrap();

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
}
