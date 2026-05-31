use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetDetailedLinks, ChromiumDatasetQuery,
        SavedChromiumDatasetRecord,
    },
    id::Id,
    library::SavedLibraryRecord,
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;
use nonempty::NonemptyString;

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
    Json(mut query): Json<ChromiumDatasetQuery>,
) -> Result<Json<Vec<ChromiumDatasetDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_datasets_detailed(&tx, state.raw_files_url(), &mut query)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_chromium_datasets_detailed(
    tx: &db::Transaction<'_>,
    raw_files_url: &str,
    query: &mut ChromiumDatasetQuery,
) -> Result<Vec<ChromiumDatasetDetailed>, ErrorInner> {
    static SELECT_DETAILED_CHROMIUM_DATASETS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    let sql = SELECT_DETAILED_CHROMIUM_DATASETS.finish_with_query(query);

    let stream = tx.query_stream(sql).await?;
    Ok(stream
        .map(|row| map_detailed_row(raw_files_url, row.unwrap()))
        .collect()
        .await)
}

fn map_detailed_row(raw_files_url: &str, row: Row) -> ChromiumDatasetDetailed {
    let record: SavedChromiumDatasetRecord = row.get("chromium_dataset");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");
    let libraries: Vec<SavedLibraryRecord> = row.get("libraries");
    let raw_file_paths: Vec<NonemptyString> = row.get("raw_file_paths");

    ChromiumDatasetDetailed {
        links: chromium_dataset_detailed_links(raw_files_url, record.id, &raw_file_paths),
        record,
        libraries: libraries.into_iter().map(library_from_record).collect(),
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        data: row.get("data"),
    }
}

fn chromium_dataset_detailed_links(
    raw_files_url: &str,
    id: Id,
    raw_file_paths: &[NonemptyString],
) -> ChromiumDatasetDetailedLinks {
    ChromiumDatasetDetailedLinks {
        simple: chromium_dataset_links(id),
        raw_files: raw_file_paths
            .iter()
            .map(|p| format!("{raw_files_url}/{p}"))
            .collect(),
    }
}
