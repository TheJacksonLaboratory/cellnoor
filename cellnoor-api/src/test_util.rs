use std::{cmp::Ordering, fmt::Debug};

use cellnoor_models::generic_query;
use diesel_async::AsyncPgConnection;
use pretty_assertions::assert_eq;

use crate::db::{self, DbConnection};

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
pub async fn test_query<SelectFn, Filter, OrderBy, Record>(
    #[builder(start_fn)] select_fn: SelectFn,
    #[builder(finish_fn)] mut db_conn: DbConnection,
    #[builder(default = generic_query::Query::<Filter, OrderBy>::default_with_no_limit())]
    db_query: generic_query::Query<Filter, OrderBy>,
    all_records: &'static [Record],
    filter: Option<fn(&&Record) -> bool>,
    sort_by: Option<fn(&&Record, &&Record) -> Ordering>,
) where
    Filter: Default,
    OrderBy: Default,
    SelectFn: AsyncFn(
        generic_query::Query<Filter, OrderBy>,
        &AsyncPgConnection,
    ) -> Result<Vec<Record>, db::Error>,
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

    let loaded_records = select_fn(db_query, &mut db_conn).await.unwrap();
    assert!(
        !loaded_records.is_empty(),
        "no records loaded from database"
    );

    let loaded_len = loaded_records.len();
    let expected_len = expected_records.len();

    assert_eq!(
        loaded_len, expected_len,
        "database query returned {loaded_len} records, but Rust function returned {expected_len}"
    );

    let loaded_records: Vec<_> = loaded_records.iter().collect();
    slices_contain_the_same_elements_the_same_number_of_times(&loaded_records, &expected_records);

    for (i, (loaded, expected)) in loaded_records.iter().zip(&expected_records).enumerate() {
        assert_eq!(
            *loaded, *expected,
            "loaded data and expected data are sorted differently (comparison failed at record \
             {i})"
        );
    }
}

pub fn slices_contain_the_same_elements_the_same_number_of_times<T: PartialEq>(
    slice1: &[T],
    slice2: &[T],
) {
    let slice1_len = slice1.len();
    let slice2_len = slice2.len();

    assert_eq!(slice1_len, slice2_len, "slices have different lengths");

    for (i, ele) in slice1.iter().enumerate() {
        let count1 = slice1.iter().filter(|x| ele == *x).count();
        let count2 = slice2.iter().filter(|x| ele == *x).count();

        assert_eq!(
            count1, count2,
            "element {i} appeared a different number of times in each slice"
        );
    }
}
