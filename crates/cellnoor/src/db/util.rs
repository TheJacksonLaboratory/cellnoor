use cellnoor_types::query::ComplexQuery;
use postgres_types::ToSql;
use uuid::Uuid;

use super as db;
use crate::error::Error;

pub async fn select_one<P, O, T>(
    tx: &db::Transaction<'_>,
    pred: P,
    select_fn: impl AsyncFn(&db::Transaction, &ComplexQuery<P, O>) -> Result<Vec<T>, Error>,
) -> Result<T, Error>
where
    O: Default,
{
    let mut records = select_fn(tx, &ComplexQuery::from_filter(pred, true)).await?;

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
    (parent_field, parent_id): (&'static str, Uuid),
    (child_field, children_ids): (&'static str, &[Uuid]),
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

pub trait ToFieldListPlaceholdersParams<const N: usize> {
    fn to_field_list_placeholders_params(&self) -> (String, String, [&(dyn ToSql + Sync); N]);
}

type FieldValuePair<'a> = (&'static str, &'a (dyn ToSql + Sync));

pub type FieldValuePairs<'a, const N: usize> = [FieldValuePair<'a>; N];

impl<'a, const N: usize> ToFieldListPlaceholdersParams<N> for FieldValuePairs<'a, N> {
    fn to_field_list_placeholders_params(&self) -> (String, String, [&(dyn ToSql + Sync); N]) {
        let fieldnames = self.map(|(field, _)| field).join(", ");

        let fieldnames = format!("({fieldnames})");

        let indices: [_; N] = std::array::from_fn(|i| format!("${}", i + 1));
        let placeholders = indices.join(", ");
        let placeholders = format!("({placeholders})");

        let bind_params = self.map(|(_, p)| p);

        (fieldnames, placeholders, bind_params)
    }
}
