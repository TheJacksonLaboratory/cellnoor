use axum::{Json, extract::State};
use cellnoor_types::{
    library::{LibraryDetailed, LibraryQuery, SavedLibraryRecord},
    suspension_pool::SavedTaggedSpecimenRecord,
};
use deadpool_postgres::tokio_postgres::Row;

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
    Json(query): Json<LibraryQuery>,
) -> Result<Json<Vec<LibraryDetailed>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_libraries_detailed(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_libraries_detailed(
    tx: &db::Transaction<'_>,
    query: &LibraryQuery,
) -> Result<Vec<LibraryDetailed>, ErrorInner> {
    static SELECT_DETAILED_LIBRARIES: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    Ok(tx
        .select_rows(&SELECT_DETAILED_LIBRARIES, query)
        .await?
        .into_iter()
        .map(map_detailed_row)
        .collect())
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
