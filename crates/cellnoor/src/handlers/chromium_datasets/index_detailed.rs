use axum::{Json, extract::State};
use cellnoor_types::{
    chromium_dataset::{ChromiumDatasetDetailed, ChromiumDatasetQuery, SavedChromiumDatasetRecord},
    library::SavedLibraryRecord,
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, BaseSqlStmt},
    error::{Error, ErrorInner},
    handlers::{
        chromium_datasets::index_compact::{chromium_dataset_links, push_distinct_on_id},
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

    let response = select_chromium_datasets_detailed(&tx, &mut query)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(super) async fn select_chromium_datasets_detailed(
    tx: &db::Transaction<'_>,
    query: &mut ChromiumDatasetQuery,
) -> Result<Vec<ChromiumDatasetDetailed>, ErrorInner> {
    push_distinct_on_id(query);

    let sql =
        BaseSqlStmt::new(include_str!("index/select_detailed.sql")).finish_with_query(query)?;

    let stream = tx.query_stream(sql).await?;
    Ok(stream
        .map(|row| row.map(map_detailed_row).unwrap())
        .collect()
        .await)
}

fn map_detailed_row(row: Row) -> ChromiumDatasetDetailed {
    let record: SavedChromiumDatasetRecord = row.get("chromium_dataset");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");
    let libraries: Vec<SavedLibraryRecord> = row.get("libraries");

    ChromiumDatasetDetailed {
        links: chromium_dataset_links(record.id),
        record,
        libraries: libraries.into_iter().map(library_from_record).collect(),
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        raw_file_paths: row.get("raw_file_paths"),
    }
}
