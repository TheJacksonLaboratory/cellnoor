use std::{collections::HashMap, str::FromStr};

use axum::extract::{Multipart, Path, State};
use cellnoor_models::{
    IdParameter,
    chromium_dataset::metrics::{
        ParsedMetricsData,
        multi_row_csv::{self},
    },
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use heck::ToSnekCase;
use serde_json::{Number, Value};
use uuid::Uuid;

use crate::{
    api::routes::chromium_datasets::files::common::{FieldExt, ParsedMultipartFormField},
    db::{self, DbConnection},
    state::AppState,
};

static ALLOWED_CONTENT_TYPES: &[&str] = &["application/json", "text/csv"];

#[axum::debug_handler]
pub async fn upload_metrics_file(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    mut request: Multipart,
) -> Result<(), db::Error> {
    let mut extracted_metrics_files = Vec::with_capacity(16);
    while let Some(field) = request
        .next_field()
        .await
        .map_err(|e| db::DataError::new_other(&e.body_text()))?
    {
        let extracted = field.parse(ALLOWED_CONTENT_TYPES).await?;
        let content = extracted.content();
        let parsed_content = if extracted.content_type() == "application/json" {
            serde_json::from_slice(content).map_err(|e| db::DataError::new_other(&e.to_string()))?
        } else {
            parse_single_row_csv(content)
                .map(ParsedMetricsData::KeyValue)
                .or_else(|_| parse_multi_row_csv(content).map(ParsedMetricsData::Tabular))?
        };

        extracted_metrics_files.push((extracted, parsed_content));
    }

    insert_file(id, &extracted_metrics_files, &db_conn).await
}

async fn insert_file(
    chromium_dataset_id: Uuid,
    data: &[(ParsedMultipartFormField, ParsedMetricsData)],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    use cellnoor_schema::chromium_dataset_metrics_files::dsl::*;

    let insertables: Vec<_> = data
        .iter()
        .map(|(form_field, parsed)| {
            (
                dataset_id.eq(chromium_dataset_id),
                path.eq(form_field.path()),
                content_type.eq(form_field.content_type()),
                raw_content.eq(form_field.content()),
                parsed_data.eq(parsed),
            )
        })
        .collect();

    diesel::insert_into(chromium_dataset_metrics_files)
        .values(&insertables)
        .execute(&mut db_conn)
        .await?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{parse_multi_row_csv, parse_single_row_csv};

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
