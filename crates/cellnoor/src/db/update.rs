use std::fmt::Write;

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
    F: Copy + Into<&'static str>,
    T: AsFieldValuePairs<F, N>,
{
    let record = data.as_field_value_pairs();
    let sql = convert_record_to_update_stmt(table, &id, &record)?;

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
) -> Result<Sql<'a>, ErrorInner>
where
    F: Copy + Into<&'static str>,
{
    fn map_err(e: std::fmt::Error) -> ErrorInner {
        ErrorInner::Other {
            message: e.to_string(),
            sql_state: None,
        }
    }

    if N == 0 {
        return Err(ErrorInner::Other {
            message: "no update provided".to_string(),
            sql_state: None,
        });
    }

    let mut params = Vec::with_capacity(N + 1);

    // Assume that `column = $n` is 32 characters at maximum, leaving room also for
    // the `update table set` part
    let mut update_clause = String::with_capacity(32 * N);
    write!(update_clause, "update {table} set ").map_err(map_err)?;

    for (i, (field, value)) in record.iter().enumerate() {
        if i != 0 {
            update_clause.push_str(", ");
        }

        let field: &str = field.clone().into();
        write!(
            update_clause,
            "{} = ${}",
            field.split(".").last().unwrap(),
            i + 1
        )
        .map_err(map_err)?;
        params.push(*value);
    }
    write!(update_clause, " where id = ${}", N + 1).map_err(map_err)?;
    params.push(id);

    Ok(Sql(update_clause, params))
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
        let sql = test_update_clause();
        let update_clause = sql.stmt();

        assert_eq!(
            update_clause,
            "update institution set name = $1, microsoft_entra_tenant_id = $2 where id = $3"
        );
    }

    #[test]
    fn params_are_correct() {
        let mut actual_params = BytesMut::new();
        let sql = test_update_clause();
        let params = sql.params();

        params[0]
            .to_sql_checked(&Type::TEXT, &mut actual_params)
            .unwrap();
        for p in &params[1..] {
            p.to_sql_checked(&Type::UUID, &mut actual_params).unwrap();
        }

        let mut expected_params = BytesMut::new();
        "name".to_sql(&Type::TEXT, &mut expected_params).unwrap();
        for uuid in [Uuid::max(), Uuid::nil()] {
            uuid.to_sql(&Type::UUID, &mut expected_params).unwrap();
        }

        assert_eq!(actual_params, expected_params)
    }
}
