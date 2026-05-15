use cellnoor_types::{ToPredicate, query::ComplexQuery};
use postgres_types::ToSql;

use super::Transaction;
use crate::error::ErrorInner;

pub async fn select_one<P, O, T>(
    tx: &Transaction<'_>,
    pred: P,
    select_fn: impl AsyncFn(&Transaction, &ComplexQuery<P, O>) -> Result<Vec<T>, ErrorInner>,
) -> Result<T, ErrorInner>
where
    O: Default,
{
    let mut records = select_fn(tx, &ComplexQuery::from_filter(pred, true)).await?;

    if records.len() != 1 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(records.swap_remove(0))
}

pub fn construct_select_stmt<'a, P, O>(
    table: &str,
    columns: &[&str],
    group_by: Option<&str>,
    ComplexQuery {
        filter,
        limit,
        offset,
        order_by,
        detailed,
    }: &'a ComplexQuery<P, O>,
) -> (String, Vec<&'a (dyn ToSql + Sync)>)
where
    P: AsRef<str> + ToPredicate,
    O: Copy + Default + AsRef<str>,
{
    let columns = columns.join(", ");

    let (where_clause, params) = if let Some(filter) = filter {
        filter.to_where_clause()
    } else {
        (String::new(), Vec::new())
    };

    let group_by = if let Some(grouping_field) = group_by {
        format!("group by {grouping_field}")
    } else {
        String::new()
    };

    let order_by = order_by.to_order_by_clause();

    let limit = if let Some(limit) = limit {
        format!("limit {limit}")
    } else {
        String::new()
    };

    let query = format!(
        "select {columns} from {table} where {where_clause} {group_by} limit {limit} offset \
         {offset}"
    );

    (query, params)
}

#[cfg(test)]
mod tests {
    use cellnoor_types::{
        StringOperator,
        institution::{InstitutionField, InstitutionPredicate, InstitutionQuery},
    };
    use deadpool_postgres::tokio_postgres::types::private::BytesMut;
    use postgres_types::{ToSql, Type};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::db::construct_select_stmt;

    fn test_select_stmt() -> (String, Vec<&'static (dyn ToSql + Sync)>) {
        construct_select_stmt(
            "institution",
            &["institution"],
            None,
            &InstitutionQuery::from_filter(
                InstitutionPredicate::Name(StringOperator::Like("institution".to_owned())),
                false,
            ),
        )
    }

    #[test]
    fn select_stmt_has_correct_sql() {
        let (select_stmt, _) = test_select_stmt();

        assert_eq!(
            select_stmt,
            "select institution from institution where (name like ($1))"
        );
    }

    #[test]
    fn params_are_correct() {
        let mut actual_params = BytesMut::new();
        let (_, params) = test_select_stmt();
        params[0].to_sql(&Type::TEXT, &mut actual_params).unwrap();

        let mut expected_params = BytesMut::new();
        "institution"
            .to_sql(&Type::TEXT, &mut expected_params)
            .unwrap();

        assert_eq!(actual_params, expected_params)
    }
}
