use uuid::Uuid;

use crate::db::{AsFieldValuePairs, FieldValuePairs, Sql};

pub async fn insert_into<F, T, const N: usize>(
    tx: &super::Transaction<'_>,
    table: &str,
    data: &T,
) -> Result<Uuid, deadpool_postgres::tokio_postgres::Error>
where
    F: Copy + Into<&'static str>,
    T: AsFieldValuePairs<F, N>,
{
    let record = data.as_field_value_pairs();
    let sql = convert_record_to_insert_stmt(table, &record, Some("id"));

    tx.query_one_into(&sql).await
}

pub async fn insert_into_no_returning<F, T, const N: usize>(
    tx: &super::Transaction<'_>,
    table: &str,
    data: &T,
) -> Result<(), deadpool_postgres::tokio_postgres::Error>
where
    F: Copy + Into<&'static str>,
    T: AsFieldValuePairs<F, N>,
{
    let record = data.as_field_value_pairs();
    let sql = convert_record_to_insert_stmt(table, &record, None);

    tx.execute(&sql).await?;

    Ok(())
}

fn convert_record_to_insert_stmt<'a, F, const N: usize>(
    table: &str,
    field_value_pairs: &'a FieldValuePairs<'a, F, N>,
    returning: Option<&str>,
) -> Sql<'a>
where
    F: Copy + Into<&'static str>,
{
    let mut fieldnames = Vec::with_capacity(N);
    let mut placeholders = Vec::with_capacity(N);
    let mut params = Vec::with_capacity(N);

    for (i, (field, value)) in field_value_pairs.iter().enumerate() {
        let field: &str = field.clone().into();

        fieldnames.push(field.split('.').last().unwrap());
        placeholders.push(format!("${}", i + 1));
        params.push(*value);
    }

    let joined_fieldnames = fieldnames.join(", ");
    let joined_placeholders = placeholders.join(", ");

    let returning = match returning {
        Some(returning) => format!("returning {returning}"),
        None => String::new(),
    };

    let insert_clause = format!(
        "insert into {table} ({joined_fieldnames}) values ({joined_placeholders}) {returning}"
    );

    Sql(insert_clause, params)
}

#[cfg(test)]
mod tests {
    use cellnoor_types::institution::InstitutionField;
    use deadpool_postgres::tokio_postgres::types::private::BytesMut;
    use postgres_types::{ToSql, Type};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::db::{Sql, insert::convert_record_to_insert_stmt};

    static TEST_DATA: [(InstitutionField, &'static (dyn ToSql + Sync)); 2] = [
        (InstitutionField::Name, &"name"),
        (InstitutionField::MicrosoftEntraTenantId, &Uuid::nil()),
    ];

    fn test_insert_stmt() -> Sql<'static> {
        convert_record_to_insert_stmt("institution", &TEST_DATA, Some("id"))
    }

    #[test]
    fn insert_stmt_has_correct_sql() {
        let Sql(insert_clause, _) = test_insert_stmt();

        assert_eq!(
            insert_clause,
            "insert into institution (name, microsoft_entra_tenant_id) values ($1, $2) returning \
             id"
        );
    }

    #[test]
    fn params_are_correct() {
        let mut actual_params = BytesMut::new();
        let Sql(_, params) = test_insert_stmt();

        params[0]
            .to_sql_checked(&Type::TEXT, &mut actual_params)
            .unwrap();
        params[1]
            .to_sql_checked(&Type::UUID, &mut actual_params)
            .unwrap();

        let mut expected_params = BytesMut::new();
        "name".to_sql(&Type::TEXT, &mut expected_params).unwrap();
        Uuid::nil()
            .to_sql(&Type::UUID, &mut expected_params)
            .unwrap();

        assert_eq!(actual_params, expected_params)
    }
}
