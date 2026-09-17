use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    library::{LibraryCompact, LibraryQuery, SavedLibraryRecord},
};

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub fn library_from_record(record: SavedLibraryRecord) -> LibraryCompact {
    LibraryCompact {
        links: SimpleLinks::from_str_and_id("/libraries", record.id),
        record,
    }
}

pub async fn index_libraries(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<LibraryQuery>,
) -> Result<Json<Vec<LibraryCompact>>, Error> {
    state
        .in_transaction(user, async |tx| select_libraries_compact(tx, &query).await)
        .await
}

async fn select_libraries_compact(
    tx: &db::Transaction<'_>,
    query: &LibraryQuery,
) -> Result<Vec<LibraryCompact>, ErrorInner> {
    static SELECT_COMPACT_LIBRARIES: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    Ok(tx
        .select(&SELECT_COMPACT_LIBRARIES, query)
        .await?
        .into_iter()
        .map(library_from_record)
        .collect())
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        library::{LibraryField, LibraryPredicateInner, LibraryQuery},
        operator::UuidOperator,
        specimen::SpecimenField,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::libraries::{
            create::test::insert_test_library, index_compact::select_libraries_compact,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_library(&tx, |_| ()).await.unwrap();

        let libraries = select_libraries_compact(
            &tx,
            &LibraryQuery::from_filter(
                LibraryPredicateInner::Id(UuidOperator::Eq(*inserted.record.id)).into(),
            ),
        )
        .await
        .unwrap();

        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].record, inserted.record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let view = "chromium_library_to_specimen";

        tokio::join!(
            ensure_fields_are_selectable::<SpecimenField>(&tx, view),
            ensure_fields_are_selectable::<LibraryField>(&tx, view),
        );
    }
}
