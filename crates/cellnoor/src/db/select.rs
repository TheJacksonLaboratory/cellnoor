use std::fmt::Write;

use cellnoor_types::{ToPredicate, query::ComplexQuery};
use postgres_types::ToSql;

use super::Transaction;
use crate::{db::SqlTemplate, error::ErrorInner};

pub async fn select_one<P, O, T>(
    tx: &Transaction<'_>,
    pred: P,
    select_fn: impl AsyncFn(&Transaction, &mut ComplexQuery<P, O>) -> Result<Vec<T>, ErrorInner>,
) -> Result<T, ErrorInner>
where
    O: Default,
{
    let mut records = select_fn(tx, &mut ComplexQuery::from_filter(pred, true)).await?;

    if records.len() != 1 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(records.swap_remove(0))
}
