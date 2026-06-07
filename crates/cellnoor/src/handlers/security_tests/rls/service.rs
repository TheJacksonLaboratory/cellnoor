use std::assert_matches;

use cellnoor_types::{
    operator::UuidOperator,
    service::{
        NewServiceRecord, Service, ServicePredicate, ServiceQuery, ServiceUpdate,
    },
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::{
        people::create::test::insert_test_person_and_institution,
        services::{
            access::add_people::insert_service_accesses, create::test::insert_test_service,
            index::select_services, update::update_service_by_id,
        },
    },
    state::test_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
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
            record: NewServiceRecord {
                description: Some("foo".to_nonempty_string()),
                can_read_all_projects: false,
                can_admin_users: false,
            },
            permissions_to_grant: None,
            permissions_to_revoke: None,
        },
    )
    .await
    .unwrap_err();

    assert_matches!(error, ErrorInner::ResourceNotFound);
}

async fn user_cannot_grant_access_to_unowned_service(client: &mut db::Client, service_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let error = insert_service_accesses(&tx, service_id, &[Uuid::new_v4()])
        .await
        .unwrap_err();

    assert_matches!(error, ErrorInner::PermissionDenied { .. });
}

async fn user_can_only_see_accessible_services(
    client: &mut db::Client,
    accessible_services: &[Service],
) {
    let tx = client.begin().await.unwrap();

    let services = select_services(&tx, &mut ServiceQuery::default())
        .await
        .unwrap();

    assert_eq!(services, accessible_services);
}

async fn user_cannot_see_inaccessible_services(client: &mut db::Client, service_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let inaccessible = select_services(
        &tx,
        &mut ServicePredicate::Id(UuidOperator::Eq(service_id)).into(),
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

    let user1_id = *user1.record.id;
    let user1_svc_acct = create_service_for(user1_id).await;

    let user2_id = *user2.record.id;
    let user2_svc_acct = create_service_for(user2_id).await;

    let mut user1_client = db_client_as_user(user1_id).await;

    user_cannot_update_unowned_service(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_grant_access_to_unowned_service(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_see_inaccessible_services(&mut user1_client, user2_svc_acct.id).await;
    user_can_only_see_accessible_services(&mut user1_client, &[user1_svc_acct.clone()]).await;

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
