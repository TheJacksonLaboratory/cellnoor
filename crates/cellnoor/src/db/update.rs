use std::fmt::Write;

use uuid::Uuid;

use crate::db::{FieldValueSlice, Sql};

/// `update <relation> set <column> = $1, … where id = $n`
pub(super) fn update_stmt<'a, F>(
    relation: &str,
    id: &'a Uuid,
    fields: &FieldValueSlice<'a, F>,
) -> Sql<'a>
where
    F: AsRef<str>,
{
    // Assume that `column = $n` is 32 characters at maximum, leaving room also
    // for the `update <relation> set` part
    let mut stmt = String::with_capacity(32 * fields.len());
    let mut params = Vec::with_capacity(fields.len() + 1);

    write!(stmt, "update {relation} set ").unwrap();

    for (i, (field, value)) in fields.iter().enumerate() {
        if i != 0 {
            stmt.push_str(", ");
        }

        params.push(*value);
        write!(stmt, "{} = ${}", field.as_ref(), params.len()).unwrap();
    }

    params.push(id);
    write!(stmt, " where id = ${}", params.len()).unwrap();

    Sql(stmt, params)
}

#[cfg(test)]
mod tests {
    use cellnoor_types::institution::InstitutionField;
    use deadpool_postgres::tokio_postgres::types::private::BytesMut;
    use postgres_types::{ToSql, Type};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::db::{Sql, update::update_stmt};

    static TEST_ID: Uuid = Uuid::nil();
    static TEST_DATA: [(InstitutionField, &'static (dyn ToSql + Sync)); 2] = [
        (InstitutionField::Name, &"name"),
        (InstitutionField::MicrosoftEntraTenantId, &Uuid::max()),
    ];

    fn test_update_stmt() -> Sql<'static> {
        update_stmt("institution", &TEST_ID, &TEST_DATA)
    }

    #[test]
    fn update_stmt_has_correct_sql() {
        let Sql(update_clause, _) = test_update_stmt();

        assert_eq!(
            update_clause,
            "update institution set name = $1, microsoft_entra_tenant_id = $2 where id = $3"
        );
    }

    #[test]
    fn params_are_correct() {
        let mut actual_params = BytesMut::new();
        let Sql(_, params) = test_update_stmt();

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
