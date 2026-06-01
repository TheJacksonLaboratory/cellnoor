use std::assert_matches;

use cellnoor_types::{
    operator::UuidOperator,
    service_account::{NewServiceAccount, ServiceAccountPredicate, ServiceAccountQuery},
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
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
async fn create_own_service_account(user: Uuid) -> Uuid {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, service_account) = insert_test_service_account(&tx, |_| ()).await.unwrap();
    let id = service_account.id;

    tx.commit().await.unwrap();

    id
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
    let user1_svc_acct = create_own_service_account(user1_id).await;

    let user2_id = *user2.record.id;
    let user2_svc_acct = create_own_service_account(user2_id).await;

    // A user cannot update another user's service account
    let mut user1_client = db_client_as_user(user1_id).await;
    let tx = user1_client.begin().await.unwrap();

    let error = update_service_account_by_id(
        &tx,
        user2_svc_acct,
        &NewServiceAccount {
            description: Some("updated".to_nonempty_string()),
            people: vec![],
        },
    )
    .await
    .unwrap_err();
    assert_matches!(error, ErrorInner::ResourceNotFound);

    // Commit the transaction because you can't reuse a transaction with a failed
    // operation in it
    tx.commit().await.unwrap();

    // A user cannot grant access to a service account they don't own
    let tx = user1_client.begin().await.unwrap();
    let error = insert_service_account_accesses(&tx, user2_svc_acct, &[user1_id])
        .await
        .unwrap_err();
    assert_matches!(error, ErrorInner::PermissionDenied { .. });
    tx.commit().await.unwrap();

    // A user cannot see a service account they haven't been granted access to
    let tx = user1_client.begin().await.unwrap();

    let inaccessible = select_service_accounts(
        &tx,
        &mut ServiceAccountPredicate::Id(UuidOperator::Eq(user2_svc_acct)).into(),
    )
    .await
    .unwrap();
    assert_eq!(inaccessible, []);

    // But they can see their own
    let accessible = select_service_accounts(
        &tx,
        &mut ServiceAccountPredicate::Id(UuidOperator::Eq(user1_svc_acct)).into(),
    )
    .await
    .unwrap();
    assert_eq!(accessible.len(), 1);

    // Now user1 grants access to user2
    insert_service_account_accesses(&tx, user1_svc_acct, &[user2_id])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut user2_client = db_client_as_user(user2_id).await;
    let tx = user2_client.begin().await.unwrap();
    let accounts = select_service_accounts(&tx, &mut ServiceAccountQuery::default())
        .await
        .unwrap();
    assert_eq!(accounts.len(), 2);
}
