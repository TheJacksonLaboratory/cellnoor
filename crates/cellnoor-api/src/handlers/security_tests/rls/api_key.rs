use std::assert_matches;

use cellnoor_types::{
    api_key::{ApiKey, ApiKeyPredicate, ApiKeyUpdate, SavedApiKeyRecord},
    bot_account::BotAccount,
    operator::UuidOperator,
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db::{self, DbError, Sql},
    handlers::{
        api_keys::{
            create::test::insert_test_api_key, index::select_api_keys, update::update_api_key_by_id,
        },
        bot_accounts::{
            access::add_people::insert_bot_account_accesses, create::test::insert_test_bot_account,
        },
        people::create::insert_test_person_and_institution,
        security_tests::permissions::create_test_user,
    },
    state::dev_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
};

async fn create_person_api_key_for(user: Uuid) -> ApiKey {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, api_key) = insert_test_api_key(&tx, |_| ()).await.unwrap();

    tx.commit().await.unwrap();

    api_key
}

async fn create_bot_account_api_key_for(user: Uuid) -> (BotAccount, ApiKey) {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, bot_account) = insert_test_bot_account(&tx, |_| ()).await.unwrap();

    let (_, api_key) = insert_test_api_key(&tx, |key| {
        key.owner_id = Some(bot_account.id);
    })
    .await
    .unwrap();

    tx.commit().await.unwrap();

    (bot_account, api_key)
}

async fn user_cannot_update_unowned_api_key(client: &mut db::Client, api_key_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let error = update_api_key_by_id(
        &tx,
        api_key_id,
        &ApiKeyUpdate {
            description: Some("updated".to_nonempty_string()),
            expires_at: None,
        },
    )
    .await
    .unwrap_err();

    assert_matches!(error, DbError::ResourceNotFound);
}

async fn user_cannot_delete_unowned_api_key(client: &mut db::Client, api_key_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let error = tx.delete::<ApiKeyUpdate>(api_key_id).await.unwrap_err();

    assert_matches!(error, DbError::ResourceNotFound);
}

async fn user_cannot_see_inaccessible_api_key(client: &mut db::Client, api_key_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let inaccessible = select_api_keys(
        &tx,
        &ApiKeyPredicate::Id(UuidOperator::Eq(api_key_id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(inaccessible, []);
}

async fn user_can_see_accessible_api_key(client: &mut db::Client, api_key: SavedApiKeyRecord) {
    let tx = client.begin().await.unwrap();

    let accessible = select_api_keys(
        &tx,
        &ApiKeyPredicate::Id(UuidOperator::Eq(api_key.id)).into(),
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

    let user1_id = user1.record.id;
    let user2_id = user2.record.id;

    let mut user2_client = db_client_as_user(user2_id).await;

    // A user can't update another user's API key
    let user1_api_key = create_person_api_key_for(user1_id).await;
    user_cannot_update_unowned_api_key(&mut user2_client, user1_api_key.record.id).await;

    // A user can't see an API key unless they've been granted access to its
    // bot_account account
    let (bot_account, bot_account_api_key) = create_bot_account_api_key_for(user1_id).await;

    user_cannot_see_inaccessible_api_key(&mut user2_client, bot_account_api_key.record.id).await;

    // Now user1 grants user2 access to the bot_account account
    let mut user1_client = db_client_as_user(user1_id).await;
    let tx = user1_client.begin().await.unwrap();
    insert_bot_account_accesses(&tx, bot_account.id, &[user2_id])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    user_can_see_accessible_api_key(&mut user2_client, bot_account_api_key.record).await;
}

// Access to a bot_account lets a user see its API keys, but only the
// bot_account's owner can change or delete them
#[tokio::test(flavor = "multi_thread")]
async fn only_bot_account_owner_can_write_bot_account_api_keys() {
    let owner_id = create_test_user().await;
    let member_id = create_test_user().await;

    let (bot_account, api_key) = create_bot_account_api_key_for(owner_id).await;
    let api_key_id = api_key.record.id;

    let mut owner_client = db_client_as_user(owner_id).await;
    let tx = owner_client.begin().await.unwrap();
    insert_bot_account_accesses(&tx, bot_account.id, &[member_id])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut member_client = db_client_as_user(member_id).await;
    user_can_see_accessible_api_key(&mut member_client, api_key.record).await;
    user_cannot_update_unowned_api_key(&mut member_client, api_key_id).await;
    user_cannot_delete_unowned_api_key(&mut member_client, api_key_id).await;

    let tx = owner_client.begin().await.unwrap();
    update_api_key_by_id(
        &tx,
        api_key_id,
        &ApiKeyUpdate {
            description: Some("updated".to_nonempty_string()),
            expires_at: None,
        },
    )
    .await
    .unwrap();
    tx.delete::<ApiKeyUpdate>(api_key_id).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn hashed_api_keys_cannot_be_read() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let error = tx
        .execute(&Sql::new("select hashed_key from api_key", vec![]))
        .await
        .unwrap_err();

    assert_eq!(
        error,
        DbError::PermissionDenied {
            message: "permission denied for resource api_key".to_owned()
        }
    );
}
