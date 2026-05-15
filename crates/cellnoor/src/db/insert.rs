use postgres_types::ToSql;
use uuid::Uuid;

use crate::db::{Record, ToRecord};

pub async fn insert_into<F, T, const N: usize>(
    tx: &super::Transaction<'_>,
    table: &str,
    data: &T,
) -> Result<Uuid, deadpool_postgres::tokio_postgres::Error>
where
    F: Copy + AsRef<str>,
    T: ToRecord<F, N>,
{
    let record = data.to_record();
    let (query, params) = convert_record_to_insert_stmt(table, &record);

    tx.query_one_into(&query, &params).await
}

fn convert_record_to_insert_stmt<'a, F, const N: usize>(
    table: &str,
    record: &'a Record<'a, F, N>,
) -> (String, Vec<&'a (dyn ToSql + Sync)>)
where
    F: Copy + AsRef<str>,
{
    let mut fieldnames = Vec::with_capacity(N);
    let mut placeholders = Vec::with_capacity(N);
    let mut params = Vec::with_capacity(N);

    for (i, (field, value)) in record.iter().enumerate() {
        fieldnames.push(field.as_ref());
        placeholders.push(format!("${}", i + 1));
        params.push(*value);
    }

    let joined_fieldnames = fieldnames.join(", ");
    let joined_placeholders = placeholders.join(", ");

    let insert_clause = format!(
        "insert into {table} ({joined_fieldnames}) values ({joined_placeholders}) returning id"
    );

    (insert_clause, params)
}

#[cfg(test)]
mod tests {
    use cellnoor_types::institution::InstitutionField;
    use deadpool_postgres::tokio_postgres::types::private::BytesMut;
    use postgres_types::{ToSql, Type};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::db::insert::convert_record_to_insert_stmt;

    fn test_insert_clause() -> (String, Vec<&'static (dyn ToSql + Sync)>) {
        convert_record_to_insert_stmt(
            "institution",
            &[
                (InstitutionField::Name, "name"),
                (InstitutionField::MicrosoftEntraTenantId, &Uuid::nil()),
            ],
        )
    }

    #[test]
    fn insert_stmt_has_correct_sql() {
        let (insert_clause, _) = test_insert_clause();

        assert_eq!(
            insert_clause,
            "insert into institution (name, microsoft_entra_id) values ($1, $2)"
        );
    }

    #[test]
    fn params_are_correct() {
        let mut actual_params = BytesMut::new();
        let (_, params) = test_insert_clause();
        params[0].to_sql(&Type::TEXT, &mut actual_params).unwrap();
        params[1].to_sql(&Type::UUID, &mut actual_params).unwrap();

        let mut expected_params = BytesMut::new();
        "name".to_sql(&Type::TEXT, &mut expected_params).unwrap();
        Uuid::nil()
            .to_sql(&Type::UUID, &mut expected_params)
            .unwrap();

        assert_eq!(actual_params, expected_params)
    }
}
