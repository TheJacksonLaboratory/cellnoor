use std::assert_matches;

use cellnoor_types::{
    api_key::{ApiKey, ApiKeyPredicate, ApiKeyRecord, ApiKeyUpdate},
    operator::UuidOperator,
    service_account::ServiceAccount,
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::ErrorInner,
    handlers::{
        api_keys::{
            create::test::insert_test_api_key, index::select_api_keys, update::update_api_key_by_id,
        },
        people::create::test::insert_test_person_and_institution,
        service_accounts::{
            access::add_people::insert_service_account_accesses,
            create::test::insert_test_service_account,
        },
    },
    state::test_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
};

async fn create_person_api_key_for(user: Uuid) -> ApiKey {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, api_key) = insert_test_api_key(&tx, AuthUser::new_as_user(user), |_| ())
        .await
        .unwrap();

    tx.commit().await.unwrap();

    api_key
}

async fn create_service_account_api_key_for(user: Uuid) -> (ServiceAccount, ApiKey) {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, service_account) = insert_test_service_account(&tx, |_| ()).await.unwrap();

    let (_, api_key) = insert_test_api_key(&tx, AuthUser::new_as_user(user), |key| {
        key.service_account_id = Some(service_account.id);
    })
    .await
    .unwrap();

    tx.commit().await.unwrap();

    (service_account, api_key)
}

async fn user_cannot_update_unowned_api_key(client: &mut db::Client, api_key_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let error = update_api_key_by_id(
        &tx,
        api_key_id,
        &ApiKeyUpdate {
            description: Some("updated".to_nonempty_string()),
            expires_at: None,
            permissions_to_grant: vec![].into(),
            permissions_to_revoke: vec![].into(),
        },
    )
    .await
    .unwrap_err();

    assert_matches!(error, ErrorInner::ResourceNotFound);
}

async fn user_cannot_see_inaccessible_api_key(client: &mut db::Client, api_key_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let inaccessible = select_api_keys(
        &tx,
        &mut ApiKeyPredicate::Id(UuidOperator::Eq(api_key_id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(inaccessible, []);
}

async fn user_can_see_accessible_api_key(client: &mut db::Client, api_key: ApiKeyRecord) {
    let tx = client.begin().await.unwrap();

    let accessible = select_api_keys(
        &tx,
        &mut ApiKeyPredicate::Id(UuidOperator::Eq(api_key.id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(accessible, [api_key]);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_api_keys() {
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
    let user2_id = *user2.record.id;

    let mut user2_client = db_client_as_user(user2_id).await;

    // A user can't update another user's API key
    let user1_api_key = create_person_api_key_for(user1_id).await;
    user_cannot_update_unowned_api_key(&mut user2_client, user1_api_key.record.id).await;

    // A user can't see an API key unless they've been granted access to its service
    // account
    let (service_account, service_account_api_key) =
        create_service_account_api_key_for(user1_id).await;

    user_cannot_see_inaccessible_api_key(&mut user2_client, service_account_api_key.record.id)
        .await;

    // Now user1 grants user2 access to the service account
    let mut user1_client = db_client_as_user(user1_id).await;
    let tx = user1_client.begin().await.unwrap();
    insert_service_account_accesses(&tx, service_account.id, &[user2_id])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    user_can_see_accessible_api_key(&mut user2_client, service_account_api_key.record).await;
}
