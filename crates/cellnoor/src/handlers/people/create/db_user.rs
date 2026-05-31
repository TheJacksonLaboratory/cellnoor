use cellnoor_types::person::{Action, PermissionsToGrant, PermissionsToRevoke, ResourcePermission};
use uuid::Uuid;

use crate::{
    db::{self, SqlBuilder},
    error::ErrorInner,
};

pub(super) async fn create_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
) -> Result<(), ErrorInner> {
    static PROVISION_USER: SqlBuilder =
        SqlBuilder::new("select create_person_user_if_not_exists($1)");

    let user_id = user_id.to_string();
    let sql = PROVISION_USER.finish_with_params(vec![&user_id]);

    tx.execute(&sql).await?;

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct UserCanCreatePerson(bool);

pub(super) async fn modify_person_permissions(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    grant_permissions: &PermissionsToGrant,
    revoke_permissions: &PermissionsToRevoke,
) -> Result<UserCanCreatePerson, ErrorInner> {
    let filter_person_creation_perms = |p: &&ResourcePermission| matches!(*p, ResourcePermission::Person(a) if a.contains(&Action::Create));

    let grant_create_person = grant_permissions
        .iter()
        .filter(filter_person_creation_perms)
        .map(Clone::clone)
        .collect::<Vec<_>>()
        .into();

    let revoke_create_person = revoke_permissions
        .iter()
        .filter(filter_person_creation_perms)
        .map(Clone::clone)
        .collect::<Vec<_>>()
        .into();

    // We pass false here because we haven't run the complete set of operations that
    // may revoke the user's 'create person' privilege. This will be remedied
    // because we run another set of grants later
    grant_permissions_to_db_user(
        tx,
        user_id,
        &grant_create_person,
        UserCanCreatePerson(false),
    )
    .await?;
    revoke_permissions_from_db_user(tx, user_id, &revoke_create_person).await?;

    let can_create_person = user_can_create_person(tx, user_id).await?;
    if can_create_person {
        tx.execute_raw_sql(&format!(r#"alter user "{user_id}" with createrole"#), &[])
            .await?;
    } else {
        tx.execute_raw_sql(&format!(r#"alter user "{user_id}" with nocreaterole"#), &[])
            .await?;
    };

    Ok(UserCanCreatePerson(can_create_person))
}

pub(super) async fn grant_permissions_to_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &PermissionsToGrant,
    can_grant_to_others: UserCanCreatePerson,
) -> Result<(), ErrorInner> {
    let grant_stmts: Vec<_> = permissions
        .iter()
        .map(|p| construct_grant_statement(user_id, p, can_grant_to_others.0))
        .collect();

    let grant_ops = grant_stmts.iter().map(|s| tx.execute_raw_sql(s, &[]));
    futures::future::try_join_all(grant_ops).await?;

    Ok(())
}

pub(super) async fn revoke_permissions_from_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &PermissionsToRevoke,
) -> Result<(), ErrorInner> {
    let revoke_stmts: Vec<_> = permissions
        .iter()
        .map(|p| construct_revoke_statement(user_id, p))
        .collect();

    let revoke_ops = revoke_stmts.iter().map(|s| tx.execute_raw_sql(s, &[]));
    futures::future::try_join_all(revoke_ops).await?;

    Ok(())
}

async fn user_can_create_person(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
) -> Result<bool, ErrorInner> {
    static SELECT_HAS_TABLE_PRIVILEGE: SqlBuilder =
        SqlBuilder::new("select has_table_privilege($1, 'person', 'insert')");

    Ok(tx
        .query_one_into(&SELECT_HAS_TABLE_PRIVILEGE.finish_with_params(vec![&user_id.to_string()]))
        .await?)
}

#[derive(Clone, Copy, strum::Display, PartialEq)]
#[strum(serialize_all = "snake_case")]
enum GrantOrRevoke {
    Grant,
    Revoke,
}

impl GrantOrRevoke {
    fn preposition(self) -> &'static str {
        match self {
            Self::Grant => "to",
            Self::Revoke => "from",
        }
    }
}

fn construct_grant_statement(
    user_id: Uuid,
    resource_permissions: &ResourcePermission,
    can_grant_to_others: bool,
) -> String {
    construct_grant_or_revoke_statement(
        GrantOrRevoke::Grant,
        user_id,
        resource_permissions,
        can_grant_to_others,
    )
}

fn construct_revoke_statement(user_id: Uuid, resource_permissions: &ResourcePermission) -> String {
    construct_grant_or_revoke_statement(GrantOrRevoke::Revoke, user_id, resource_permissions, false)
}

fn construct_grant_or_revoke_statement(
    grant_or_revoke: GrantOrRevoke,
    user_id: Uuid,
    resource_permissions: &ResourcePermission,
    with_grant: bool,
) -> String {
    let resource_names = permission_as_tableset(resource_permissions);
    let actions = match resource_permissions {
        ResourcePermission::Institution(a)
        | ResourcePermission::Person(a)
        | ResourcePermission::Project(a)
        | ResourcePermission::Specimen(a)
        | ResourcePermission::AssayConstantData(a)
        | ResourcePermission::ChromiumExperimentalData(a)
        | ResourcePermission::ChromiumDataset(a) => a,
    };

    let actions: Vec<_> = actions.iter().map(Action::as_str).collect();
    let actions = actions.join(", ");

    let suffix = if with_grant && grant_or_revoke == GrantOrRevoke::Grant {
        "with grant option"
    } else {
        ""
    };

    format!(
        r#"{grant_or_revoke} {actions} on {resource_names} {} "{user_id}" {suffix}"#,
        grant_or_revoke.preposition()
    )
}

fn permission_as_tableset(permission: &ResourcePermission) -> &'static str {
    match permission {
        ResourcePermission::Institution(_) => "institution",
        ResourcePermission::Person(_) => "person",
        ResourcePermission::Project(_) => "project",
        ResourcePermission::Specimen(_) => "specimen",
        ResourcePermission::AssayConstantData(_) => {
            "tenx_assay, index_kit, single_index_set, dual_index_set, library_type_specification, \
             multiplexing_tag"
        }
        ResourcePermission::ChromiumExperimentalData(_) => {
            "suspension, suspension_measurement, suspension_preparer, suspension_pool, \
             suspension_pool_measurement, suspension_pool_preparer, chromium_run, gem_well, \
             chip_loading, cdna, cdna_measurement, cdna_preparer, library, library_measurement, \
             library_preparer"
        }
        ResourcePermission::ChromiumDataset(_) => {
            "chromium_dataset, chromium_dataset_raw_file, chromium_dataset_parsed_file, \
             chromium_dataset_library"
        }
    }
}
