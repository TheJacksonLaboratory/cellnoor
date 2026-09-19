use std::{assert_matches, fmt::Debug};

use cellnoor_types::{
    permission::{Action, Permission, Resource},
    project::ProjectQuery,
    query::{ComplexQuery, OrderField},
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    handlers::{
        cdna::index_detailed::select_cdna_detailed,
        chromium_datasets::index_detailed::select_chromium_datasets_detailed,
        chromium_runs::index_detailed::select_chromium_runs_detailed,
        institutions::{create::test::insert_test_institution, index::select_institutions},
        libraries::index_detailed::select_libraries_detailed,
        people::{create::test::insert_test_person_and_institution, index::select_people},
        permissions::{grant_permissions, revoke_permissions},
        projects::{create::test::insert_test_project, index_detailed::select_projects_detailed},
        services::index::select_services,
        specimens::index_detailed::select_specimens_detailed,
        suspension_pools::index_detailed::select_suspension_pools_detailed,
        suspensions::index_detailed::select_suspensions_detailed,
    },
    state::test_util::{db_client_as_admin, db_client_as_user},
};

async fn create_test_user() -> Uuid {
    create_test_user_with(false, Vec::new()).await
}

async fn create_test_user_with(is_staff: bool, permissions: Vec<Permission>) -> Uuid {
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

fn may(resource: Resource, actions: &[Action]) -> Vec<Permission> {
    actions
        .iter()
        .map(|&action| Permission { resource, action })
        .collect()
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
        assert_is_ok(tx, select_projects_detailed),
        assert_is_ok(tx, select_services),
        assert_is_ok(tx, select_specimens_detailed),
        assert_is_ok(tx, select_suspensions_detailed),
        assert_is_ok(tx, select_suspension_pools_detailed),
    );
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
async fn user_with_permission_can_write() {
    let user_id = create_test_user_with(false, may(Resource::Institution, &[Action::Create])).await;

    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    insert_test_institution(&tx, |_| ()).await.unwrap();
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
        false,
        [
            may(Resource::Person, &[Action::Create, Action::Update]),
            may(Resource::Institution, &[Action::Create]),
        ]
        .concat(),
    )
    .await;
    let target_id = create_test_user().await;

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
    let user_id = create_test_user_with(false, may(Resource::Institution, &[Action::Create])).await;
    let target_id = create_test_user().await;

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
    let user_id = create_test_user_with(false, may(Resource::Institution, &[Action::Create])).await;

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
