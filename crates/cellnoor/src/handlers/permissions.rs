use cellnoor_types::{
    Relation,
    permission::{Action, Permission, Resource},
};
use uuid::Uuid;

use crate::{
    db::{self, FieldValues, Insert, SqlBuilder},
    error::ErrorInner,
};

/// One row of the `permission` table, which holds one resource and action per
/// row.
struct PermissionRow {
    principal_id: Uuid,
    permission: Permission,
}

impl Relation for PermissionRow {
    const NAME: &'static str = "permission";
}

impl Insert for PermissionRow {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            principal_id,
            permission,
        } = self;

        vec![
            ("principal_id", principal_id),
            ("resource", &permission.resource),
            ("action", &permission.action),
        ]
    }
}

fn permission_rows(principal_id: Uuid, permissions: &[Permission]) -> Vec<PermissionRow> {
    permissions
        .iter()
        .map(|&permission| PermissionRow {
            principal_id,
            permission,
        })
        .collect()
}

pub(crate) async fn grant_permissions(
    tx: &db::Transaction<'_>,
    principal_id: Uuid,
    permissions: &[Permission],
) -> Result<(), ErrorInner> {
    // Row-level security decides whether the current user may hand each of
    // these out (see db/migrations/0028_principal-rls.up.sql). Granting a
    // permission that the principal already holds is not an error
    tx.insert_many_on_conflict_do_nothing(&permission_rows(principal_id, permissions))
        .await
}

pub(crate) async fn revoke_permissions(
    tx: &db::Transaction<'_>,
    principal_id: Uuid,
    permissions: &[Permission],
) -> Result<(), ErrorInner> {
    static REVOKE_PERMISSIONS: SqlBuilder = SqlBuilder::new(
        "delete from permission where principal_id = $1 and (resource, action) in (select * from \
         unnest($2::text[], $3::text[]))",
    );

    // A delete matches a set of rows rather than adding them, so the
    // permissions go down as two parallel arrays
    let (resources, actions): (Vec<Resource>, Vec<Action>) =
        permissions.iter().map(|p| (p.resource, p.action)).unzip();

    tx.execute(&REVOKE_PERMISSIONS.finish_with_params(vec![&principal_id, &resources, &actions]))
        .await?;

    Ok(())
}
