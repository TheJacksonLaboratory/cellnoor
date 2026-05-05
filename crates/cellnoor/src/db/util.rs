use cellnoor_types::{DbQuery, Filter};

use super as db;
use crate::error::Error;

pub trait FromRecord<T> {
    fn from_record(record: T) -> Self;
}

pub async fn select_one<P, O, T>(
    tx: &db::Transaction<'_>,
    pred: P,
    select_fn: impl AsyncFn(&db::Transaction, &DbQuery<Filter<P>, O>) -> Result<Vec<T>, Error>,
) -> Result<T, Error>
where
    O: Default,
{
    let mut records = select_fn(tx, &DbQuery::from_filter(pred, true)).await?;

    if records.len() != 1 {
        return Err(Error::resource_not_found());
    }

    Ok(records.swap_remove(0))
}
