use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetDetailedLinks, ChromiumDatasetQuery,
        SavedChromiumDatasetRecord,
    },
    id::Id,
    library::SavedLibraryRecord,
    nonempty::NonemptyString,
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::{
        chromium_datasets::index_compact::chromium_dataset_links,
        libraries::index_compact::library_from_record,
        suspension_pools::index_compact::tagged_specimen_from_record,
    },
    state::AppState,
};

pub async fn index_chromium_datasets_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<ChromiumDatasetQuery>,
) -> Result<Json<Vec<ChromiumDatasetDetailed>>, Error> {
    state
        .in_transaction(user, async |tx| {
            select_chromium_datasets_detailed(tx, &state.public_files_url, &query).await
        })
        .await
}

// Visibility required for tests
pub(in super::super) async fn select_chromium_datasets_detailed(
    tx: &db::Transaction<'_>,
    raw_files_url: &str,
    query: &ChromiumDatasetQuery,
) -> Result<Vec<ChromiumDatasetDetailed>, ErrorInner> {
    static SELECT_DETAILED_CHROMIUM_DATASETS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select_rows(&SELECT_DETAILED_CHROMIUM_DATASETS, query)
        .await?
        .into_iter()
        .map(|row| map_detailed_row(raw_files_url, row))
        .collect())
}

fn map_detailed_row(raw_files_url: &str, row: Row) -> ChromiumDatasetDetailed {
    let record: SavedChromiumDatasetRecord = row.get("chromium_dataset");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");
    let libraries: Vec<SavedLibraryRecord> = row.get("libraries");
    let raw_file_paths: Vec<NonemptyString> = row.get("raw_file_paths");

    ChromiumDatasetDetailed {
        links: chromium_dataset_detailed_links(raw_files_url, record.id, &raw_file_paths),
        record,
        assay: row.get("assay"),
        libraries: libraries.into_iter().map(library_from_record).collect(),
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        data: row.get("data"),
    }
}

fn chromium_dataset_detailed_links(
    public_files_url: &str,
    id: Id,
    raw_file_paths: &[NonemptyString],
) -> ChromiumDatasetDetailedLinks {
    ChromiumDatasetDetailedLinks {
        simple: chromium_dataset_links(id),
        file_dir: format!("{public_files_url}/chromium-datasets/{id}"),
        raw_files: raw_file_paths
            .iter()
            .map(|p| format!("{public_files_url}/chromium-datasets/{id}/{p}"))
            .collect(),
    }
}
