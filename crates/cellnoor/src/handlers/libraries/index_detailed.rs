use axum::{Json, extract::State};
use cellnoor_types::{
    library::{LibraryDetailed, LibraryQuery, SavedLibraryRecord},
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::{
        libraries::index_compact::library_simple_links,
        suspension_pools::index_compact::tagged_specimen_from_record,
    },
    state::AppState,
};

pub async fn index_libraries_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<LibraryQuery>,
) -> Result<Json<Vec<LibraryDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_libraries_detailed(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_libraries_detailed(
    tx: &db::Transaction<'_>,
    query: &mut LibraryQuery,
) -> Result<Vec<LibraryDetailed>, ErrorInner> {
    static SELECT_DETAILED_LIBRARIES: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    let sql = SELECT_DETAILED_LIBRARIES.finish_with_query(query);

    let stream = tx.query_stream(sql).await?;
    Ok(stream
        .map(|row| row.map(map_detailed_row).unwrap())
        .collect()
        .await)
}

fn map_detailed_row(row: Row) -> LibraryDetailed {
    let record: SavedLibraryRecord = row.get("library");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");

    LibraryDetailed {
        links: library_simple_links(record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(tagged_specimen_from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
}
