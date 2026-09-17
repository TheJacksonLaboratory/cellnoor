use cellnoor_types::{
    Relation,
    permission::{Action, Permission, Resource},
};
use uuid::Uuid;

use crate::db::{self, DbError, FieldValues, Insert, Sql};

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
            ("action", &permission.action),
            ("resource", &permission.resource),
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
) -> Result<(), DbError> {
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
) -> Result<(), DbError> {
    static REVOKE_PERMISSIONS: &str = "delete from permission where principal_id = $1 and action \
                                       = any($2) and resource = any($3)";

    // A delete matches a set of rows rather than adding them, so the
    // permissions go down as two parallel arrays
    let (actions, resources): (Vec<Action>, Vec<Resource>) =
        permissions.iter().map(|p| (p.action, p.resource)).unzip();

    tx.execute(&Sql::new(
        REVOKE_PERMISSIONS,
        vec![&principal_id, &actions, &resources],
    ))
    .await?;

    Ok(())
}
