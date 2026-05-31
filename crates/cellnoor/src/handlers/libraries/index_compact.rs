use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    id::Id,
    library::{
        LibraryCompact, LibraryPredicate, LibraryPredicateInner, LibraryQuery, SavedLibraryRecord,
    },
};
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub(super) fn library_simple_links(id: Id) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/libraries", id)
}

pub fn library_from_record(record: SavedLibraryRecord) -> LibraryCompact {
    LibraryCompact {
        links: library_simple_links(record.id),
        record,
    }
}

pub async fn index_libraries(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<LibraryQuery>,
) -> Result<Json<Vec<LibraryCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_libraries_compact(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_libraries_compact(
    tx: &db::Transaction<'_>,
    query: &mut LibraryQuery,
) -> Result<Vec<LibraryCompact>, ErrorInner> {
    static SELECT_COMPACT_LIBRARIES: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    let sql = SELECT_COMPACT_LIBRARIES.finish_with_query(query);

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(library_from_record).collect().await)
}

impl AsPredicate for LibraryPredicate {
    fn as_predicate(&self) -> (&'static str, (&'static str, &(dyn ToSql + Sync))) {
        let sql = match self {
            Self::Specimen(p) => return p.as_predicate(),
            Self::Library(field) => match field {
                LibraryPredicateInner::Id(u) | LibraryPredicateInner::CdnaId(u) => {
                    u.as_sql_operator_and_value()
                }
                LibraryPredicateInner::ReadableId(s)
                | LibraryPredicateInner::SingleIndexSetName(s)
                | LibraryPredicateInner::DualIndexSetName(s) => s.as_sql_operator_and_value(),
                LibraryPredicateInner::NumberOfSampleIndexPcrCycles(i) => {
                    i.as_sql_operator_and_value()
                }
                LibraryPredicateInner::TargetReadsPerCell(i) => i.as_sql_operator_and_value(),
                LibraryPredicateInner::PreparedAt(t) => t.as_sql_operator_and_value(),
                LibraryPredicateInner::AdditionalData(j) => j.as_sql_operator_and_value(),
            },
        };

        (self.field_name(), sql)
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        library::{LibraryPredicateInner, LibraryQuery},
        operator::UuidOperator,
    };
    use pretty_assertions::assert_eq;

    use crate::{
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
            &mut LibraryQuery::from_filter(
                LibraryPredicateInner::Id(UuidOperator::Eq(*inserted.record.id)).into(),
            ),
        )
        .await
        .unwrap();

        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].record, inserted.record);
    }
}
