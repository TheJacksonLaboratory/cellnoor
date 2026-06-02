use crate::db;

pub async fn ensure_fields_are_selectable<F>(tx: &db::Transaction<'_>, view: &str)
where
    F: strum::VariantArray + Into<&'static str> + Copy,
{
    let specimen_fields: Vec<&str> = F::VARIANTS.iter().copied().map(Into::into).collect();
    let stmt = format!("select {} from {view}", specimen_fields.join(","));

    tx.execute_raw_sql(&stmt, &[]).await.unwrap();
}
