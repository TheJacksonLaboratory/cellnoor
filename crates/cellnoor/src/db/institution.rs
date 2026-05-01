use cellnoor_types::institution::{Institution, NewInstitution};

use crate::db;

pub async fn insert_institution(
    NewInstitution {
        name,
        microsoft_entra_tenant_id,
    }: NewInstitution,
    db_client: &mut db::Client,
) -> Result<Institution, crate::error::Error> {
    let tx = db_client.begin().await?;

    // Simple queries can be written inline
    let institution = tx
        .query_one_scalar(
            "insert into institution (name, microsoft_entra_tenant_id) values ($1, $2) returning \
             institution",
            &[&name, &microsoft_entra_tenant_id],
        )
        .await?;

    Ok(institution)
}

pub async fn select_institutions() {}
