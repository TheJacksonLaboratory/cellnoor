use uuid::Uuid;

use crate::error::ErrorInner;

pub async fn delete_by_id(
    tx: &super::Transaction<'_>,
    table: &str,
    id: Uuid,
) -> Result<(), ErrorInner> {
    let stmt = format!("delete from {table} where id = $1");

    let n = tx.execute(&stmt, &[&id]).await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(())
}
