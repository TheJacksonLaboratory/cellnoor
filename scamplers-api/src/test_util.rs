use std::{cmp::Ordering, fmt::Debug};

use diesel::{Connection, PgConnection};

use crate::db;

#[bon::builder]
fn filter_and_sort<Record>(
    data: Vec<Record>,
    filter: Option<fn(&Record) -> bool>,
    sort_by: Option<fn(&Record, &Record) -> Ordering>,
) -> Vec<Record> {
    fn identity_filter<M>(_: &M) -> bool {
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
    #[builder(finish_fn)] pooled_db_conn: deadpool_diesel::postgres::Connection,
    #[builder(default)] db_query: Query,
    all_data: Vec<Record>,
    filter: Option<fn(&Record) -> bool>,
    sort_by: Option<fn(&Record, &Record) -> Ordering>,
) where
    Query: 'static + db::Operation<Vec<Record>> + Default + Send,
    Record: 'static + Debug + PartialEq + Send + Sync,
{
    let data = filter_and_sort()
        .data(all_data)
        .maybe_filter(filter)
        .maybe_sort_by(sort_by)
        .call();

    assert_ne!(data.len(), 0, "no records found after data was filtered");

    let perform_test = move |db_conn: &mut PgConnection| {
        db_conn.test_transaction::<_, db::Error, _>(|tx| {
            let loaded_records = db_query.execute(tx).unwrap();

            assert_ne!(loaded_records.len(), 0, "no records loaded from database");

            assert_eq!(
                loaded_records.len(),
                data.len(),
                "filter returned different number of records"
            );

            for loaded in &loaded_records {
                assert!(data.contains(loaded));
            }

            for expected in &data {
                assert!(loaded_records.contains(expected));
            }

            for (i, (loaded, expected)) in loaded_records.iter().zip(&data).enumerate() {
                assert_eq!(loaded, expected, "comparison failed at record {i}");
            }

            Ok(())
        });
    };

    pooled_db_conn.interact(perform_test).await.unwrap();
}
