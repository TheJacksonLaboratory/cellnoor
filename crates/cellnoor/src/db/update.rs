use postgres_types::ToSql;
use uuid::Uuid;

use crate::{
    db::{AsFieldValuePairs, FieldValuePairs, Sql},
    error::ErrorInner,
};

pub async fn update<F, T, const N: usize>(
    tx: &super::Transaction<'_>,
    table: &str,
    id: Uuid,
    data: &T,
) -> Result<(), ErrorInner>
where
    F: Copy + AsRef<str>,
    T: AsFieldValuePairs<F, N>,
{
    let record = data.as_field_value_pairs();
    let Some(sql) = convert_record_to_update_stmt(table, &id, &record) else {
        return Ok(());
    };

    let n = tx.execute(&sql).await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(())
}

fn convert_record_to_update_stmt<'a, F, const N: usize>(
    table: &str,
    id: &'a Uuid,
    record: &'a FieldValuePairs<F, N>,
) -> Option<Sql<'a>>
where
    F: Copy + AsRef<str>,
{
    if N == 0 {
        return None;
    }

    let mut column_sets = Vec::with_capacity(N);
    let mut params = Vec::with_capacity(N + 1);

    for (i, (field, value)) in record.iter().enumerate() {
        column_sets.push(format!(
            "{} = ${}",
            field.as_ref().split('.').last().unwrap(),
            i + 1
        ));
        params.push(*value);
    }

    params.push(id);

    let column_sets = column_sets.join(", ");

    let update_clause = format!("update {table} set {column_sets} where id = ${}", N + 1);

    Some(Sql(update_clause, params))
}

#[cfg(test)]
mod tests {
    use cellnoor_types::institution::InstitutionField;
    use deadpool_postgres::tokio_postgres::types::private::BytesMut;
    use postgres_types::{ToSql, Type};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::db::{Sql, update::convert_record_to_update_stmt};

    static TEST_ID: Uuid = Uuid::nil();
    static TEST_DATA: [(InstitutionField, &'static (dyn ToSql + Sync)); 2] = [
        (InstitutionField::Name, &"name"),
        (InstitutionField::MicrosoftEntraTenantId, &Uuid::max()),
    ];

    fn test_update_clause() -> Sql<'static> {
        convert_record_to_update_stmt("institution", &TEST_ID, &TEST_DATA).unwrap()
    }

    #[test]
    fn update_stmt_has_correct_sql() {
        let Sql(update_clause, _) = test_update_clause();

        assert_eq!(
            update_clause,
            "update institution set name = $1, microsoft_entra_tenant_id = $2 where id = $3"
        );
    }

    #[test]
    fn params_are_correct() {
        let mut actual_params = BytesMut::new();
        let Sql(_, params) = test_update_clause();

        params[0]
            .to_sql_checked(&Type::TEXT, &mut actual_params)
            .unwrap();
        params[1]
            .to_sql_checked(&Type::UUID, &mut actual_params)
            .unwrap();
        params[2]
            .to_sql_checked(&Type::UUID, &mut actual_params)
            .unwrap();

        let mut expected_params = BytesMut::new();
        "name".to_sql(&Type::TEXT, &mut expected_params).unwrap();
        for uuid in [Uuid::max(), Uuid::nil()] {
            uuid.to_sql(&Type::UUID, &mut expected_params).unwrap();
        }

        assert_eq!(actual_params, expected_params)
    }
}
