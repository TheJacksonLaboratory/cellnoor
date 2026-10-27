use std::assert_matches;

use cellnoor_types::{
    operator::UuidOperator,
    permission::{Action, Resource},
    service::{Service, ServicePredicate, ServiceQuery, ServiceSimpleFields, ServiceUpdate},
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    handlers::{
        institutions::create::insert_test_institution,
        people::create::insert_test_person_and_institution,
        permissions::{grant_permissions, revoke_permissions},
        security_tests::permissions::{create_test_user, create_test_user_with, may},
        services::{
            access::{
                add_people::insert_service_accesses, remove_people::delete_service_accesses,
            },
            create::test::insert_test_service,
            index::select_services,
            index_detailed::select_services_detailed,
            update::update_service_by_id,
        },
        set_is_staff,
    },
    state::dev_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
};

// Creates a service account owned by `user`: the database default sets
// `owned_by` to the caller.
async fn create_service_for(user: Uuid) -> Service {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, service) = insert_test_service(&tx, |_| ()).await.unwrap();

    tx.commit().await.unwrap();

    service
}

async fn user_cannot_update_unowned_service(client: &mut db::Client, service_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let error = update_service_by_id(
        &tx,
        service_id,
        &ServiceUpdate {
            description: Some("foo".to_nonempty_string()),
            is_staff: false,
        },
    )
    .await
    .unwrap_err();

    assert_matches!(error, DbError::ResourceNotFound);
}

async fn user_cannot_grant_access_to_unowned_service(client: &mut db::Client, service_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let error = insert_service_accesses(&tx, service_id, &[Uuid::new_v4()])
        .await
        .unwrap_err();

    assert_matches!(error, DbError::PermissionDenied { .. });
}

async fn user_can_only_see_accessible_services(
    client: &mut db::Client,
    accessible_services: &[Service],
) {
    let tx = client.begin().await.unwrap();

    let services = select_services(&tx, &ServiceQuery::default())
        .await
        .unwrap();

    assert_eq!(services, accessible_services);
}

async fn user_cannot_see_inaccessible_services(client: &mut db::Client, service_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let inaccessible = select_services(
        &tx,
        &ServicePredicate::Id(UuidOperator::Eq(service_id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(inaccessible, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_services() {
    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();

    let (_, user1) = insert_test_person_and_institution(&tx, |_| ())
        .await
        .unwrap();
    let (_, user2) = insert_test_person_and_institution(&tx, |_| ())
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let user1_id = user1.record.id;
    let user1_svc_acct = create_service_for(user1_id).await;

    let user2_id = user2.record.id;
    let user2_svc_acct = create_service_for(user2_id).await;

    let mut user1_client = db_client_as_user(user1_id).await;

    user_cannot_update_unowned_service(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_grant_access_to_unowned_service(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_see_inaccessible_services(&mut user1_client, user2_svc_acct.id).await;
    user_can_only_see_accessible_services(&mut user1_client, std::slice::from_ref(&user1_svc_acct))
        .await;

    // Now user1 grants access to user2
    let tx = user1_client.begin().await.unwrap();
    insert_service_accesses(&tx, user1_svc_acct.id, &[user2_id])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut user2_client = db_client_as_user(user2_id).await;
    user_can_only_see_accessible_services(&mut user2_client, &[user2_svc_acct, user1_svc_acct])
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn service_owner_cannot_make_service_staff() {
    let owner_id = create_test_user().await;
    let service = create_service_for(owner_id).await;

    let mut client = db_client_as_user(owner_id).await;

    // The owner may update the service's principal, so the failure below comes
    // from the staff check
    let tx = client.begin().await.unwrap();
    set_is_staff(&tx, service.id, false).await.unwrap();
    drop(tx);

    let tx = client.begin().await.unwrap();

    assert_eq!(
        set_is_staff(&tx, service.id, true).await.unwrap_err(),
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"principal\"".to_owned()
        }
    );
}

// Owning a service lets a user grant it the permissions they hold, even though
// they can't update people. That doesn't extend to services they don't own
#[tokio::test(flavor = "multi_thread")]
async fn service_owner_can_only_grant_permissions_to_owned_service() {
    let permissions = may(Resource::Institution, &[Action::Create]);

    let owner_id = create_test_user_with(false, permissions.clone()).await;
    let owned_service = create_service_for(owner_id).await;
    let unowned_service = create_service_for(create_test_user().await).await;

    let mut client = db_client_as_user(owner_id).await;

    // This needs its own transaction because the failure aborts it
    let tx = client.begin().await.unwrap();
    let error = grant_permissions(&tx, unowned_service.id, &permissions)
        .await
        .unwrap_err();

    assert_matches!(error, DbError::PermissionDenied { .. });
    drop(tx);

    let tx = client.begin().await.unwrap();
    grant_permissions(&tx, owned_service.id, &permissions)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Using the permission would need the service to be staff, so check that it
    // holds it
    let tx = client.begin().await.unwrap();
    let service = tx
        .select_one(
            ServicePredicate::Id(UuidOperator::Eq(owned_service.id)),
            select_services_detailed,
        )
        .await
        .unwrap();

    assert_eq!(service.permissions, permissions);
}

// `update` on people doesn't let a user change the permissions of a service
// they don't own
#[tokio::test(flavor = "multi_thread")]
async fn person_updater_cannot_change_permissions_of_unowned_service() {
    let create_institution = may(Resource::Institution, &[Action::Create]);
    let update_institution = may(Resource::Institution, &[Action::Update]);

    let owner_id = create_test_user_with(false, create_institution.clone()).await;
    let service = create_service_for(owner_id).await;

    let mut client = db_client_as_user(owner_id).await;
    let tx = client.begin().await.unwrap();
    grant_permissions(&tx, service.id, &create_institution)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // They are staff and hold every permission involved, so only ownership can
    // refuse them
    let updater_id = create_test_user_with(
        true,
        [
            may(Resource::Person, &[Action::Update]),
            create_institution.clone(),
            update_institution.clone(),
        ]
        .concat(),
    )
    .await;

    let mut client = db_client_as_user(updater_id).await;

    // This needs its own transaction because the failure aborts it
    let tx = client.begin().await.unwrap();
    assert_matches!(
        grant_permissions(&tx, service.id, &update_institution)
            .await
            .unwrap_err(),
        DbError::PermissionDenied { .. }
    );
    drop(tx);

    let tx = client.begin().await.unwrap();
    assert_eq!(
        revoke_permissions(&tx, service.id, &create_institution)
            .await
            .unwrap_err(),
        DbError::ResourceNotFound
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn service_cannot_change_itself() {
    let service = create_service_for(create_test_user().await).await;

    let mut client = db_client_as_user(service.id).await;

    user_cannot_update_unowned_service(&mut client, service.id).await;
    user_cannot_grant_access_to_unowned_service(&mut client, service.id).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn service_owner_can_update_staff_service_without_changing_staff_status() {
    let owner_id = create_test_user().await;
    let service = create_service_for(owner_id).await;

    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();
    set_is_staff(&tx, service.id, true).await.unwrap();
    tx.commit().await.unwrap();

    let mut client = db_client_as_user(owner_id).await;
    let tx = client.begin().await.unwrap();

    set_is_staff(&tx, service.id, true).await.unwrap();
}
