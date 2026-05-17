use uuid::Uuid;

use crate::{db::Sql, error::ErrorInner};

pub async fn delete_by_id(
    tx: &super::Transaction<'_>,
    table: &str,
    id: Uuid,
) -> Result<(), ErrorInner> {
    let stmt = format!("delete from {table} where id = $1");

    let n = tx.execute(&Sql(stmt, vec![&id])).await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(())
}
