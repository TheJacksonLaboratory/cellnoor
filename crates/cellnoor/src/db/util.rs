use cellnoor_types::{DbQuery, Filter, person::ResourcePermission};
use postgres_types::ToSql;
use uuid::Uuid;

use super as db;
use crate::error::Error;

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

#[derive(Clone, Copy, Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum JunctionTable {
    ProjectAccess,
    SuspensionPreparer,
    SuspensionPoolPreparer,
    CdnaPreparer,
    LibraryPreparer,
    ChromiumDatasetLibrary,
}

pub async fn insert_many_to_many(
    tx: &db::Transaction<'_>,
    table: JunctionTable,
    (parent_field, parent_id): (&str, Uuid),
    (child_field, children_ids): (&str, &[Uuid]),
) -> Result<(), Error> {
    if children_ids.is_empty() {
        return Ok(());
    }

    let params: Vec<&(dyn ToSql + Sync)> = children_ids
        .iter()
        .flat_map(|child| {
            [
                &parent_id as &(dyn ToSql + Sync),
                child as &(dyn ToSql + Sync),
            ]
        })
        .collect();
    let mut values_clause = String::with_capacity(64);

    let mut current_param_number = 1;
    for (i, _) in params.chunks(2).enumerate() {
        values_clause.push_str(&format!(
            "(${current_param_number}, ${})",
            current_param_number + 1
        ));

        if i != (params.len() / 2) - 1 {
            values_clause.push(',');
        }

        current_param_number += 2;
    }

    let stmt = format!(
        "insert into {table} ({parent_field}, {child_field}) values {values_clause} on conflict \
         do nothing"
    );

    tx.execute(&stmt, &params).await?;

    Ok(())
}
