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
        detailed: _,
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
        "select {columns} from {table} {where_clause} {group_by} {order_by} {limit} offset \
         {offset}"
    );

    (query, params)
}

#[cfg(test)]
mod tests {
    use cellnoor_types::{
        institution::{InstitutionField, InstitutionPredicate, InstitutionQuery},
        operator::StringOperator,
        order_by::{OrderBy, OrderBySet},
    };
    use deadpool_postgres::tokio_postgres::types::private::BytesMut;
    use postgres_types::{ToSql, Type};
    use pretty_assertions::assert_eq;

    use crate::db::construct_select_stmt;

    #[test]
    fn select_stmt_has_correct_sql() {
        let mut q = InstitutionQuery::from_filter(
            InstitutionPredicate::Name(StringOperator::Like("institution".to_owned())),
            false,
        );
        q.limit = Some(1);
        q.order_by = OrderBySet::Many(vec![
            OrderBy {
                field: InstitutionField::Id,
                desc: false,
            },
            OrderBy {
                field: InstitutionField::Name,
                desc: true,
            },
        ]);

        let (select_stmt, _) = construct_select_stmt("institution", &["institution"], None, &q);

        assert_eq!(
            select_stmt,
            "select institution from institution where (institution).name like ($1)  order by \
             (institution).id asc, (institution).name desc limit 1 offset 0"
        );
    }

    #[test]
    fn params_are_correct() {
        let mut actual_params = BytesMut::new();

        let q = InstitutionQuery::from_filter(
            InstitutionPredicate::Name(StringOperator::Like("institution".to_owned())),
            false,
        );
        let (_, params) = construct_select_stmt("institution", &["institution"], None, &q);

        params[0]
            .to_sql_checked(&Type::TEXT, &mut actual_params)
            .unwrap();

        let mut expected_params = BytesMut::new();
        "institution"
            .to_sql(&Type::TEXT, &mut expected_params)
            .unwrap();

        assert_eq!(actual_params, expected_params)
    }
}
