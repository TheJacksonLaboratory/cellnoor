use std::fmt::Write;

use postgres_types::ToSql;

use crate::db::{ColumnSlice, Columns, Sql};

/// `insert into <relation> (<columns>) values ($1, …) [returning <returning>]`
pub(super) fn insert_stmt<'a>(
    relation: &str,
    fields: &ColumnSlice<'a>,
    returning: Option<&str>,
) -> Sql<'a> {
    // Should be more than enough space
    let mut stmt = String::with_capacity(512);
    let mut params = Vec::with_capacity(fields.len());

    write!(stmt, "insert into {relation} (").unwrap();
    write_columns(&mut stmt, fields);
    stmt.push_str(") values (");
    write_placeholders(&mut stmt, &mut params, fields);
    stmt.push(')');

    if let Some(returning) = returning {
        write!(stmt, " returning {returning}").unwrap();
    }

    Sql(stmt, params)
}

/// `insert into <relation> (<columns>) values ($1, …), ($2, …)`
///
/// Every row names the same columns, so the first one decides the column list.
/// `rows` must not be empty.
pub(super) fn insert_many_stmt<'a>(
    relation: &str,
    rows: &[Columns<'a>],
    on_conflict_do_nothing: bool,
) -> Sql<'a> {
    let first_row = &rows[0];

    let mut stmt = String::with_capacity(512);
    let mut params = Vec::with_capacity(rows.len() * first_row.len());

    write!(stmt, "insert into {relation} (").unwrap();
    write_columns(&mut stmt, first_row);
    stmt.push_str(") values ");

    for (i, row) in rows.iter().enumerate() {
        if i != 0 {
            stmt.push_str(", ");
        }

        stmt.push('(');
        write_placeholders(&mut stmt, &mut params, row);
        stmt.push(')');
    }

    if on_conflict_do_nothing {
        stmt.push_str(" on conflict do nothing");
    }

    Sql(stmt, params)
}

fn write_columns(stmt: &mut String, fields: &ColumnSlice<'_>) {
    for (i, (field, _)) in fields.iter().enumerate() {
        if i != 0 {
            stmt.push_str(", ");
        }

        stmt.push_str(field);
    }
}

fn write_placeholders<'a>(
    stmt: &mut String,
    params: &mut Vec<&'a (dyn ToSql + Sync)>,
    fields: &ColumnSlice<'a>,
) {
    for (i, (_, value)) in fields.iter().enumerate() {
        if i != 0 {
            stmt.push_str(", ");
        }

        params.push(*value);
        write!(stmt, "${}", params.len()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use deadpool_postgres::tokio_postgres::types::private::BytesMut;
    use postgres_types::{ToSql, Type};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::db::{
        Sql,
        insert::{insert_many_stmt, insert_stmt},
    };

    static TEST_DATA: [(&str, &'static (dyn ToSql + Sync)); 2] = [
        ("name", &"name"),
        ("microsoft_entra_tenant_id", &Uuid::nil()),
    ];

    fn test_insert_stmt() -> Sql<'static> {
        insert_stmt("institution", &TEST_DATA, Some("id"))
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
    fn insert_many_stmt_has_correct_sql() {
        let rows = vec![TEST_DATA.to_vec(), TEST_DATA.to_vec()];
        let Sql(insert_clause, params) = insert_many_stmt("institution", &rows, true);

        assert_eq!(
            insert_clause,
            "insert into institution (name, microsoft_entra_tenant_id) values ($1, $2), ($3, $4) \
             on conflict do nothing"
        );
        assert_eq!(params.len(), 4);
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
