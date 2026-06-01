use std::assert_matches;

use cellnoor_types::{
    operator::UuidOperator,
    service_account::{
        NewServiceAccount, ServiceAccount, ServiceAccountPredicate, ServiceAccountQuery,
    },
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::{
        people::create::test::insert_test_person_and_institution,
        service_accounts::{
            access::add_people::insert_service_account_accesses,
            create::test::insert_test_service_account, index::select_service_accounts,
            update::update_service_account_by_id,
        },
    },
    state::test_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
};

// Creates a service account owned by `user`: the database default sets
// `owned_by` to the caller.
async fn create_service_account_for(user: Uuid) -> ServiceAccount {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, service_account) = insert_test_service_account(&tx, |_| ()).await.unwrap();

    tx.commit().await.unwrap();

    service_account
}

async fn user_cannot_update_unowned_service_account(
    client: &mut db::Client,
    service_account_id: Uuid,
) {
    let tx = client.begin().await.unwrap();

    let error = update_service_account_by_id(
        &tx,
        service_account_id,
        &NewServiceAccount {
            description: Some("updated".to_nonempty_string()),
            users: vec![],
        },
    )
    .await
    .unwrap_err();

    assert_matches!(error, ErrorInner::ResourceNotFound);
}

async fn user_cannot_grant_access_to_unowned_service_account(
    client: &mut db::Client,
    service_account_id: Uuid,
) {
    let tx = client.begin().await.unwrap();

    let error = insert_service_account_accesses(&tx, service_account_id, &[Uuid::new_v4()])
        .await
        .unwrap_err();

    assert_matches!(error, ErrorInner::PermissionDenied { .. });
}

async fn user_can_only_see_accessible_service_accounts(
    client: &mut db::Client,
    accessible_service_accounts: &[ServiceAccount],
) {
    let tx = client.begin().await.unwrap();

    let service_accounts = select_service_accounts(&tx, &mut ServiceAccountQuery::default())
        .await
        .unwrap();

    assert_eq!(service_accounts, accessible_service_accounts);
}

async fn user_cannot_see_inaccessible_service_accounts(
    client: &mut db::Client,
    service_account_id: Uuid,
) {
    let tx = client.begin().await.unwrap();

    let inaccessible = select_service_accounts(
        &tx,
        &mut ServiceAccountPredicate::Id(UuidOperator::Eq(service_account_id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(inaccessible, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_service_accounts() {
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
    let user1_svc_acct = create_service_account_for(user1_id).await;

    let user2_id = *user2.record.id;
    let user2_svc_acct = create_service_account_for(user2_id).await;

    let mut user1_client = db_client_as_user(user1_id).await;

    user_cannot_update_unowned_service_account(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_grant_access_to_unowned_service_account(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_see_inaccessible_service_accounts(&mut user1_client, user2_svc_acct.id).await;
    user_can_only_see_accessible_service_accounts(&mut user1_client, &[user1_svc_acct.clone()])
        .await;

    // Now user1 grants access to user2
    let tx = user1_client.begin().await.unwrap();
    insert_service_account_accesses(&tx, user1_svc_acct.id, &[user2_id])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut user2_client = db_client_as_user(user2_id).await;
    user_can_only_see_accessible_service_accounts(
        &mut user2_client,
        &[user2_svc_acct, user1_svc_acct],
    )
    .await;
}
