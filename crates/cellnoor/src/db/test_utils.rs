use crate::db;

pub async fn ensure_fields_are_selectable<F>(tx: &db::Transaction<'_>, view: &str)
where
    F: strum::VariantArray + AsRef<str> + Copy,
{
    let fields: Vec<&str> = F::VARIANTS.iter().map(AsRef::as_ref).collect();
    let stmt = format!("select {} from {view}", fields.join(","));

    tx.execute_raw_sql(&stmt, &[]).await.unwrap();
}
