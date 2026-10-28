use cellnoor_types::query::Field;

use crate::db::{Sql, Transaction};

pub async fn ensure_fields_are_selectable<F>(tx: &Transaction<'_>, view: &str)
where
    F: Field + strum::VariantArray,
{
    let fields: Vec<String> = F::VARIANTS
        .iter()
        .map(|field| format!("({}).{}", F::RELATION, field.as_ref()))
        .collect();
    let stmt = format!("select {} from {view}", fields.join(","));

    tx.execute(&Sql(stmt, Vec::new())).await.unwrap();
}
