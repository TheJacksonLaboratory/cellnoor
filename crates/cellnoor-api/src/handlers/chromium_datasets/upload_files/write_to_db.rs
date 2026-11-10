use std::collections::HashMap;

use cellnoor_types::{Relation, chromium_dataset::ChromiumDatasetParsedFileData};
use csvranger::TenxCsvValue;
use uuid::Uuid;

use crate::{
    db::{self, DbError, FieldValues, Insert, Sql},
    handlers::chromium_datasets::upload_files::{
        UploadFilesError,
        allowed_content_type::AllowedContentType,
        stage_files::{FileHash, StagedFile},
        upload_path::UploadPath,
    },
};

pub(super) async fn write_files_to_db(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    staged_files: &[StagedFile],
) -> Result<Vec<String>, DbError> {
    let overwritten_file_paths = delete_files_with_same_paths(tx, dataset_id, staged_files).await?;
    // Ensure the raw file is inserted first because the parsed file depends on
    // the raw file's existence
    insert_raw_files(tx, dataset_id, staged_files).await?;
    insert_parsed_files(tx, dataset_id, staged_files).await?;

    Ok(overwritten_file_paths)
}

// `path` is case-insensitive, so the returned paths can differ from the staged
// files' paths in letter case. Parsed files are deleted along with their raw
// file
async fn delete_files_with_same_paths(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    staged_files: &[StagedFile],
) -> Result<Vec<String>, DbError> {
    static DELETE_RAW_FILES: &str = "delete from chromium_dataset_raw_file where dataset_id = $1 \
                                     and path = any($2) returning path";

    let staged_file_paths: Vec<_> = staged_files
        .iter()
        .map(|staged_file| staged_file.path_relative_to_dataset_directory.as_str())
        .collect();

    tx.query_into(&Sql::new(
        DELETE_RAW_FILES,
        vec![&dataset_id, &staged_file_paths],
    ))
    .await
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
    staged_files: &[StagedFile],
) -> Result<(), DbError> {
    let records: Vec<_> = staged_files
        .iter()
        .map(|staged_file| NewRawFile {
            dataset_id,
            path: &staged_file.path_relative_to_dataset_directory,
            hash: &staged_file.hash,
        })
        .collect();

    tx.insert_many(&records).await
}

struct NewRawFile<'a> {
    dataset_id: Uuid,
    path: &'a UploadPath,
    hash: &'a FileHash,
}

impl Relation for NewRawFile<'_> {
    const NAME: &'static str = "chromium_dataset_raw_file";
}

impl Insert for NewRawFile<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            dataset_id,
            path,
            hash,
        } = self;

        vec![
            ("dataset_id", dataset_id),
            ("path", path),
            ("rapidhash_v3_hash", hash),
        ]
    }
}

async fn insert_parsed_files(
    tx: &db::Transaction<'_>,
    dataset_id: Uuid,
    staged_files: &[StagedFile],
) -> Result<(), DbError> {
    let records: Vec<_> = staged_files
        .iter()
        .filter_map(|staged_file| {
            staged_file
                .parsed_data
                .as_ref()
                .map(|parsed_data| NewParsedFile {
                    dataset_id,
                    path: &staged_file.path_relative_to_dataset_directory,
                    data: postgres_types::Json(parsed_data),
                })
        })
        .collect();

    tx.insert_many(&records).await
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

// Visibility required for tests
pub(in super::super) fn parse_csv(
    data: &[u8],
) -> Result<ChromiumDatasetParsedFileData, UploadFilesError> {
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rapidhash::v3::rapidhash_v3;
    use uuid::Uuid;

    use super::{StagedFile, UploadPath, write_files_to_db};
    use crate::{
        db::Sql,
        handlers::{
            chromium_datasets::{
                create::insert_test_chromium_dataset, upload_files::stage_files::FileHash,
            },
            chromium_runs::create::TestChromiumRunKind,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn file_whose_path_differs_in_case_replaces_existing_file() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, ds) = insert_test_chromium_dataset(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        let dataset_id = *ds.record.id;

        let staged_file_with_path = |path_relative_to_dataset_directory| StagedFile {
            path_relative_to_dataset_directory: UploadPath::new(Some(
                path_relative_to_dataset_directory,
            ))
            .unwrap(),
            parsed_data: None,
            hash: FileHash::new(rapidhash_v3(Uuid::new_v4().as_bytes())),
        };

        let lowercase_path = "per_sample_outs/sample1/summary.csv";
        let uppercase_path = "per_sample_outs/SAMPLE1/summary.csv";

        let overwritten_file_paths =
            write_files_to_db(&tx, dataset_id, &[staged_file_with_path(lowercase_path)])
                .await
                .unwrap();
        assert!(overwritten_file_paths.is_empty());

        let overwritten_file_paths =
            write_files_to_db(&tx, dataset_id, &[staged_file_with_path(uppercase_path)])
                .await
                .unwrap();
        assert_eq!(overwritten_file_paths, [lowercase_path]);

        let stored_file_paths: Vec<String> = tx
            .query_into(&Sql::new(
                "select path from chromium_dataset_raw_file where dataset_id = $1",
                vec![&dataset_id],
            ))
            .await
            .unwrap();

        assert_eq!(stored_file_paths, [uppercase_path]);
    }
}
