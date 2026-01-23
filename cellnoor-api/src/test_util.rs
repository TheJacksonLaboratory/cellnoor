use std::{cmp::Ordering, fmt::Debug};

use diesel_async::{AsyncConnection, AsyncPgConnection, scoped_futures::ScopedFutureExt};
use pretty_assertions::assert_eq;

use crate::{
    api,
    db::{self, DbConnection},
    test_state::DefaultWithNoLimit,
};

#[bon::builder]
fn filter_and_sort<Record>(
    data: &[Record],
    filter: Option<fn(&&Record) -> bool>,
    sort_by: Option<fn(&&Record, &&Record) -> Ordering>,
) -> Vec<&Record>
where
    Record: 'static,
{
    fn identity_filter<M>(_: &&M) -> bool {
        true
    }

    let filter = filter.unwrap_or(identity_filter);

    let mut data: Vec<_> = data.into_iter().filter(filter).collect();

    if let Some(compare) = sort_by {
        data.sort_by(compare);
    }

    data
}

#[bon::builder]
#[builder(finish_fn = run)]
pub async fn test_query<Query, Record>(
    #[builder(finish_fn)] mut db_conn: DbConnection,
    #[builder(default = Query::default_with_no_limit())] db_query: Query,
    all_records: &'static [Record],
    filter: Option<fn(&&Record) -> bool>,
    sort_by: Option<fn(&&Record, &&Record) -> Ordering>,
) where
    Query: 'static + api::Request<Vec<Record>> + DefaultWithNoLimit + Send,
    Record: 'static + Debug + PartialEq + Send + Sync,
{
    let expected_records = filter_and_sort()
        .data(all_records)
        .maybe_filter(filter)
        .maybe_sort_by(sort_by)
        .call();

    assert!(
        !expected_records.is_empty(),
        "no records found after data was filtered"
    );

    db_conn
        .test_transaction::<_, db::Error, _>(|tx| {
            async {
            let loaded_records = db_query.handle_without_authorization(tx).await;

            assert!(
                !loaded_records.is_empty(),
                "no records loaded from database"
            );

            let loaded_len = loaded_records.len();
            let expected_len = expected_records.len();

            assert_eq!(
                loaded_len, expected_len,
                "database query returned {loaded_len} records, but Rust function returned \
             {expected_len}"
            );

            for loaded in &loaded_records {
                assert!(expected_records.contains(&loaded));
            }

            for expected in &expected_records {
                assert!(loaded_records.contains(*expected));
            }

            for (i, (loaded, expected)) in loaded_records.iter().zip(&expected_records).enumerate()
            {
                assert_eq!(
                    loaded, *expected,
                    "loaded data and expected data are sorted differently (comparison failed at \
                 record {i})"
                );
            }

            Ok(())
        }
        .scope_boxed()
        })
        .await;
}
