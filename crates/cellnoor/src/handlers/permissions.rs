use cellnoor_types::permission::{Action, Permission, Resource};
use uuid::Uuid;

use crate::{db, error::ErrorInner};

/// The `permission` table holds one row per resource and action, so a set of
/// permissions goes to the database as two parallel arrays.
fn resources_and_actions(permissions: &[Permission]) -> (Vec<Resource>, Vec<Action>) {
    permissions.iter().map(|p| (p.resource, p.action)).unzip()
}

pub(crate) async fn grant_permissions(
    tx: &db::Transaction<'_>,
    principal_id: Uuid,
    permissions: &[Permission],
) -> Result<(), ErrorInner> {
    let (resources, actions) = resources_and_actions(permissions);

    // Row-level security decides whether the current user may hand each of
    // these out (see db/migrations/0028_principal-rls.up.sql)
    tx.execute_raw_sql(
        "insert into permission (principal_id, resource, action) select $1, * from \
         unnest($2::text[], $3::text[]) on conflict do nothing",
        &[&principal_id, &resources, &actions],
    )
    .await?;

    Ok(())
}

pub(crate) async fn revoke_permissions(
    tx: &db::Transaction<'_>,
    principal_id: Uuid,
    permissions: &[Permission],
) -> Result<(), ErrorInner> {
    let (resources, actions) = resources_and_actions(permissions);

    tx.execute_raw_sql(
        "delete from permission where principal_id = $1 and (resource, action) in (select * from \
         unnest($2::text[], $3::text[]))",
        &[&principal_id, &resources, &actions],
    )
    .await?;

    Ok(())
}
