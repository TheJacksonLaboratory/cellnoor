use std::{assert_matches, fmt::Debug};

use cellnoor_types::{
    permission::{Action, Permission, Resource},
    person::PersonUpdate,
    project::ProjectQuery,
    query::{ComplexQuery, OrderField},
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db::{self, DbError, Sql},
    handlers::{
        bot_accounts::{index::select_bot_accounts, index_detailed::select_bot_accounts_detailed},
        cdna::index_detailed::select_cdna_detailed,
        chromium_datasets::index_detailed::select_chromium_datasets_detailed,
        chromium_runs::index_detailed::select_chromium_runs_detailed,
        institutions::{create::insert_test_institution, index::select_institutions},
        libraries::index_detailed::select_libraries_detailed,
        people::{
            PersonError, create::insert_test_person_and_institution, index::select_people,
            index_permissions::select_people_with_permissions, update::update_person_by_id,
        },
        permissions::{grant_permissions, revoke_permissions},
        projects::{create::insert_test_project, index_detailed::select_projects_detailed},
        security_tests::rls::specimen::get_user_id_from_specimen,
        specimens::{
            create::insert_test_specimen_and_project, index_detailed::select_specimens_detailed,
        },
        suspension_pools::index_detailed::select_suspension_pools_detailed,
        suspensions::index_detailed::select_suspensions_detailed,
    },
    state::dev_util::{db_client_as_admin, db_client_as_user},
};

pub async fn create_test_user() -> Uuid {
    create_test_user_with(false, Vec::new()).await
}

pub async fn create_test_user_with(is_staff: bool, permissions: Vec<Permission>) -> Uuid {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (_, person) = insert_test_person_and_institution(&tx, |p| {
        p.simple.is_staff = is_staff;
        p.permissions_to_grant = permissions.clone();
    })
    .await
    .unwrap();

    tx.commit().await.unwrap();

    person.record.id
}

pub fn may(resource: Resource, actions: &[Action]) -> Vec<Permission> {
    actions
        .iter()
        .map(|&action| Permission { resource, action })
        .collect()
}

// Adding a child row (a measurement, a preparer) needs `create` or `update` on
// its resource, and removing one needs `delete` or `update`. Only staff may do
// either. `add` and `remove` each change one of the parent's children, and
// `member_id` can see the parent
pub async fn assert_only_staff_with_permission_can_change_children<A, R, E>(
    resource: Resource,
    member_id: Uuid,
    add: A,
    remove: R,
) where
    A: AsyncFn(&db::Transaction) -> Result<(), DbError>,
    R: AsyncFn(&db::Transaction) -> Result<(), E>,
    E: From<DbError> + PartialEq + Debug,
{
    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();

    grant_permissions(
        &tx,
        member_id,
        &may(resource, &[Action::Create, Action::Update, Action::Delete]),
    )
    .await
    .unwrap();

    tx.commit().await.unwrap();

    let staff_without_permission = create_test_user_with(true, Vec::new()).await;
    let staff_with_create = create_test_user_with(true, may(resource, &[Action::Create])).await;
    let staff_with_update = create_test_user_with(true, may(resource, &[Action::Update])).await;
    let staff_with_delete = create_test_user_with(true, may(resource, &[Action::Delete])).await;

    // The member holds every permission but isn't staff, and the other user is
    // staff without any permission
    for user_id in [member_id, staff_without_permission] {
        let mut client = db_client_as_user(user_id).await;

        // Each attempt needs its own transaction because a failure aborts it
        let tx = client.begin().await.unwrap();
        assert_matches!(
            add(&tx).await.unwrap_err(),
            DbError::PermissionDenied { .. }
        );
        drop(tx);

        let tx = client.begin().await.unwrap();
        assert_eq!(
            remove(&tx).await.unwrap_err(),
            DbError::ResourceNotFound.into()
        );
    }

    let mut client = db_client_as_user(staff_with_create).await;
    let tx = client.begin().await.unwrap();
    add(&tx).await.unwrap();
    drop(tx);

    // Committed so that `remove` sees the added child, which matters for
    // preparers because at least one has to remain
    let mut client = db_client_as_user(staff_with_update).await;
    let tx = client.begin().await.unwrap();
    add(&tx).await.unwrap();
    tx.commit().await.unwrap();

    // Each removal is rolled back, so bot_accounth remove the same rows
    for user_id in [staff_with_delete, staff_with_update] {
        let mut client = db_client_as_user(user_id).await;
        let tx = client.begin().await.unwrap();
        remove(&tx).await.unwrap();
    }
}

async fn assert_is_ok<F, Pred, Order, Ret>(tx: &db::Transaction<'_>, select_fn: F)
where
    Order: OrderField,
    F: AsyncFn(&db::Transaction, &ComplexQuery<Pred, Order>) -> Result<Ret, DbError>,
    Ret: Debug,
{
    assert_matches!(select_fn(tx, &ComplexQuery::default()).await, Ok(_));
}

#[tokio::test(flavor = "multi_thread")]
async fn user_can_access_every_view() {
    let user_id = create_test_user().await;

    let mut client = db_client_as_user(user_id).await;
    let tx = &client.begin().await.unwrap();

    tokio::join!(
        assert_is_ok(tx, select_cdna_detailed),
        assert_is_ok(tx, async |tx, q| select_chromium_datasets_detailed(tx, q)
            .await),
        assert_is_ok(tx, select_chromium_runs_detailed),
        assert_is_ok(tx, select_institutions),
        assert_is_ok(tx, select_libraries_detailed),
        assert_is_ok(tx, select_people),
        assert_is_ok(tx, select_people_with_permissions),
        assert_is_ok(tx, select_projects_detailed),
        assert_is_ok(tx, select_bot_accounts),
        assert_is_ok(tx, select_bot_accounts_detailed),
        assert_is_ok(tx, select_specimens_detailed),
        assert_is_ok(tx, select_suspensions_detailed),
        assert_is_ok(tx, select_suspension_pools_detailed),
    );
}

async fn count_rows(tx: &db::Transaction<'_>, table: &str) -> i64 {
    tx.query_one(&Sql::new(
        &format!("select count(*) from {table}"),
        Vec::new(),
    ))
    .await
    .unwrap()
    .get(0)
}

// These tables are scoped to project members themselves. Everything downstream
// of a specimen is scoped by the views that read it, which the `rls` tests
// check. Committee approvals have no rows here, so they aren't checked
#[tokio::test(flavor = "multi_thread")]
async fn project_tables_are_visible_only_to_project_members() {
    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();

    let (_, specimen) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
    let member_id = get_user_id_from_specimen(&tx, &specimen).await;

    tx.commit().await.unwrap();

    let tables = [
        "project",
        "project_access",
        "specimen",
        "specimen_measurement",
    ];

    let mut client = db_client_as_user(create_test_user().await).await;
    let tx = client.begin().await.unwrap();

    for table in tables {
        assert_eq!(count_rows(&tx, table).await, 0, "{table}");
    }

    let mut client = db_client_as_user(member_id).await;
    let tx = client.begin().await.unwrap();

    for table in tables {
        assert_ne!(count_rows(&tx, table).await, 0, "{table}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn user_without_permission_cannot_write() {
    let user_id = create_test_user().await;

    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    let error = insert_test_institution(&tx, |_| ()).await.unwrap_err();

    assert_matches!(error, DbError::PermissionDenied { .. });
}

#[tokio::test(flavor = "multi_thread")]
async fn staff_with_permission_can_write() {
    let user_id = create_test_user_with(true, may(Resource::Institution, &[Action::Create])).await;

    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    insert_test_institution(&tx, |_| ()).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn nonstaff_cannot_write_even_with_permission() {
    let user_id = create_test_user_with(false, may(Resource::Institution, &[Action::Create])).await;

    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    assert_matches!(
        insert_test_institution(&tx, |_| ()).await.unwrap_err(),
        DbError::PermissionDenied { .. }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn staff_can_see_projects_they_are_not_a_member_of() {
    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();

    let (_, project) = insert_test_project(&tx, |p| p.members = vec![])
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let staff_id = create_test_user_with(true, Vec::new()).await;
    let mut client = db_client_as_user(staff_id).await;
    let tx = client.begin().await.unwrap();

    let visible = select_projects_detailed(&tx, &ProjectQuery::default())
        .await
        .unwrap();

    assert!(visible.contains(&project));
}

// A user can only hand out a permission they hold themselves, which is what
// stops the `person` permission from being a way to grant anything at all
#[tokio::test(flavor = "multi_thread")]
async fn user_can_only_grant_permissions_they_hold() {
    let manager_id = create_test_user_with(
        true,
        [
            may(Resource::Person, &[Action::Create, Action::Update]),
            may(Resource::Institution, &[Action::Create]),
        ]
        .concat(),
    )
    .await;
    let target_id = create_test_user_with(true, Vec::new()).await;

    let mut client = db_client_as_user(manager_id).await;

    // The manager can't create specimens, so they can't let anyone else either.
    // This needs its own transaction because the failure aborts it
    let tx = client.begin().await.unwrap();
    let error = grant_permissions(&tx, target_id, &may(Resource::Specimen, &[Action::Create]))
        .await
        .unwrap_err();

    assert_matches!(error, DbError::PermissionDenied { .. });
    drop(tx);

    // But they can pass on the permission they do hold
    let tx = client.begin().await.unwrap();
    grant_permissions(
        &tx,
        target_id,
        &may(Resource::Institution, &[Action::Create]),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut client = db_client_as_user(target_id).await;
    let tx = client.begin().await.unwrap();

    insert_test_institution(&tx, |_| ()).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn user_who_cannot_update_people_cannot_grant_anything() {
    let user_id = create_test_user_with(true, may(Resource::Institution, &[Action::Create])).await;
    let target_id = create_test_user_with(true, Vec::new()).await;

    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    grant_permissions(
        &tx,
        target_id,
        &may(Resource::Institution, &[Action::Create]),
    )
    .await
    .unwrap_err();

    drop(tx);

    // Nothing was granted, so the target still can't write
    let mut client = db_client_as_user(target_id).await;
    let tx = client.begin().await.unwrap();

    assert_eq!(
        insert_test_institution(&tx, |_| ()).await.unwrap_err(),
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"institution\""
                .to_owned()
        }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn revoked_permission_disallows_writing() {
    let user_id = create_test_user_with(true, may(Resource::Institution, &[Action::Create])).await;

    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();

    revoke_permissions(&tx, user_id, &may(Resource::Institution, &[Action::Create]))
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    assert_matches!(
        insert_test_institution(&tx, |_| ()).await.unwrap_err(),
        DbError::PermissionDenied { .. }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn user_cannot_revoke_permissions_they_do_not_hold() {
    let manager_id = create_test_user_with(
        true,
        [
            may(Resource::Person, &[Action::Update]),
            may(Resource::Specimen, &[Action::Create]),
        ]
        .concat(),
    )
    .await;
    let target_permissions = [
        may(Resource::Institution, &[Action::Create]),
        may(Resource::Specimen, &[Action::Create]),
    ]
    .concat();
    let target_id = create_test_user_with(false, target_permissions.clone()).await;

    let mut client = db_client_as_user(manager_id).await;
    let tx = client.begin().await.unwrap();

    // The manager holds only one of these, so the whole revoke fails
    let error = revoke_permissions(&tx, target_id, &target_permissions)
        .await
        .unwrap_err();

    assert_eq!(error, DbError::ResourceNotFound);
}

#[tokio::test(flavor = "multi_thread")]
async fn nonstaff_cannot_update_people() {
    let user_id = create_test_user_with(false, may(Resource::Person, &[Action::Update])).await;

    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();

    let (new_target, target) = insert_test_person_and_institution(&tx, |_| ())
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    let update = PersonUpdate {
        simple: new_target.simple,
        email: None,
    };

    assert_matches!(
        update_person_by_id(&tx, target.record.id, &update)
            .await
            .unwrap_err(),
        PersonError::Db(DbError::ResourceNotFound)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn nonstaff_cannot_grant_permissions_to_people() {
    let permission = may(Resource::Institution, &[Action::Create]);

    // They hold every permission involved, so only not being staff can refuse
    // them
    let manager_id = create_test_user_with(
        false,
        [may(Resource::Person, &[Action::Update]), permission.clone()].concat(),
    )
    .await;
    let target_id = create_test_user().await;

    let mut client = db_client_as_user(manager_id).await;
    let tx = client.begin().await.unwrap();

    assert_matches!(
        grant_permissions(&tx, target_id, &permission)
            .await
            .unwrap_err(),
        DbError::PermissionDenied { .. }
    );
}
