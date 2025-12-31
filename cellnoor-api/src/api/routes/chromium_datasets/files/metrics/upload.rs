use std::{collections::HashMap, str::FromStr};

use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use cellnoor_models::chromium_dataset::{
    ChromiumDatasetIdMetrics,
    metrics::{
        ParsedMetricsData,
        multi_row_csv::{self},
    },
};
use diesel::prelude::*;
use heck::ToSnekCase;
use serde_json::{Number, Value};

use crate::{
    api::{
        self,
        extract::auth::AuthenticatedUser,
        routes::{
            ApiResponse,
            chromium_datasets::files::common::{FieldExt, ParsedMultipartFormField},
            inner_handler,
        },
    },
    db,
    state::AppState,
};

static ALLOWED_CONTENT_TYPES: &[&str] = &["application/json", "text/csv"];

pub async fn upload_metrics_file(
    chromium_dataset_id: ChromiumDatasetIdMetrics,
    state: State<AppState>,
    user: AuthenticatedUser,
    mut request: Multipart,
) -> ApiResponse<()> {
    let mut extracted_metrics_files = Vec::with_capacity(16);
    while let Some(field) = request.next_field().await? {
        let extracted = field.parse(ALLOWED_CONTENT_TYPES).await?;
        let content = extracted.content();
        let parsed_content = if extracted.content_type() == "application/json" {
            serde_json::from_slice(content).map_err(|e| api::ErrorResponse {
                status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                public_error: api::Error::MalformedRequest {
                    message: format!("error parsing JSON: {e}"),
                },
                internal_error: None,
            })?
        } else {
            parse_single_row_csv(content)
                .map(ParsedMetricsData::KeyValue)
                .or_else(|_| parse_multi_row_csv(content).map(ParsedMetricsData::Tabular))
                .map_err(|e| api::ErrorResponse {
                    status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                    public_error: e,
                    internal_error: None,
                })?
        };

        extracted_metrics_files.push((extracted, parsed_content));
    }

    let _ = inner_handler(state, user, (chromium_dataset_id, extracted_metrics_files)).await?;
    Ok((StatusCode::CREATED, Json(())))
}

impl db::Operation<()>
    for (
        ChromiumDatasetIdMetrics,
        Vec<(ParsedMultipartFormField, ParsedMetricsData)>,
    )
{
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<(), db::Error> {
        use cellnoor_schema::chromium_dataset_metrics_files::dsl::*;

        let (ds_id, data) = self;
        let insertables: Vec<_> = data
            .iter()
            .map(|(form_field, parsed)| {
                (
                    dataset_id.eq(ds_id),
                    directory.eq(form_field.directory()),
                    filename.eq(form_field.filename()),
                    content_type.eq(form_field.content_type()),
                    raw_content.eq(form_field.content()),
                    parsed_data.eq(parsed),
                )
            })
            .collect();

        diesel::insert_into(chromium_dataset_metrics_files)
            .values(&insertables)
            .execute(db_conn)?;

        Ok(())
    }
}

fn parse_single_row_csv(raw_content: &[u8]) -> Result<HashMap<String, Value>, api::Error> {
    let mut csv = csv::Reader::from_reader(raw_content);

    let header = csv.headers()?;
    let header_len = header.len();
    let snake_case_header: Vec<String> = header.iter().map(snake_case_field_name).collect();
    let mut records = csv.records();

    let n_rows_err = Err(api::Error::MalformedRequest {
        message: "expected exactly one row in CSV".to_owned(),
    });

    let first_record = match records.next() {
        Some(rec) => rec?,
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
        parsed_data.insert(
            field_name,
            parse_str_as_number(field_value)
                .map_err(|e| api::Error::MalformedRequest {
                    message: format!("failed to parse string as number: {e}"),
                })?
                .into(),
        );
    }

    Ok(parsed_data)
}

fn parse_multi_row_csv(raw_content: &[u8]) -> Result<Vec<multi_row_csv::Row>, api::Error> {
    let mut csv = csv::Reader::from_reader(raw_content);

    let headers = csv.headers()?.clone();

    let mut parsed_data = Vec::with_capacity(100);
    for record in csv.records() {
        let record = record?;

        let simple_fields: multi_row_csv::SimpleFields = record.deserialize(Some(&headers))?;

        let metric_value_str = record.get(5).ok_or(api::Error::MalformedRequest {
            message: "failed to parse multi-row CSV: column 'Metric Value' is missing".to_string(),
        })?;
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
