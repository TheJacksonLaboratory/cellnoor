use std::collections::HashMap;

use cellnoor_types::{Relation, chromium_dataset::ChromiumDatasetParsedFileData};
use csvranger::TenxCsvValue;
use uuid::Uuid;

use crate::{
    db::{self, DbError, FieldValues, Insert},
    handlers::chromium_datasets::upload_files::{
        UploadFilesError, allowed_content_type::AllowedContentType, upload_path::UploadPath,
    },
};

pub(super) async fn write_files_to_db(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    parsed_files: &[(UploadPath, Option<ChromiumDatasetParsedFileData>)],
) -> Result<(), DbError> {
    // Ensure the raw file is inserted first because the parsed file depends on
    // the raw file's existence
    insert_raw_files(tx, dataset_id, parsed_files).await?;
    insert_parsed_files(tx, dataset_id, parsed_files).await
}

pub(super) fn parse_file(
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
            path: p,
        })
        .collect();

    tx.insert_many(&files).await?;

    Ok(())
}

struct NewRawFile<'a> {
    dataset_id: Uuid,
    path: &'a UploadPath,
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
                path,
                data: postgres_types::Json(d),
            })
        })
        .collect();

    tx.insert_many(&records).await?;

    Ok(())
}

struct NewParsedFile<'a> {
    dataset_id: Uuid,
    path: &'a UploadPath,
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
