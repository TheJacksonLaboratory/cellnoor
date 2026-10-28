use std::assert_matches;

use cellnoor_types::{
    bot_account::{BotAccount, BotAccountPredicate, BotAccountQuery, BotAccountUpdate},
    operator::UuidOperator,
    permission::{Action, Resource},
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    handlers::{
        bot_accounts::{
            access::add_people::insert_bot_account_accesses, create::test::insert_test_bot_account,
            index::select_bot_accounts, index_detailed::select_bot_accounts_detailed,
            update::update_bot_account_by_id,
        },
        people::create::insert_test_person_and_institution,
        permissions::{grant_permissions, revoke_permissions},
        security_tests::permissions::{create_test_user, create_test_user_with, may},
        set_is_staff,
    },
    state::dev_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
};

// Creates a bot_account account owned by `user`: the database default sets
// `owned_by` to the caller.
async fn create_bot_account_for(user: Uuid) -> BotAccount {
    let mut client = db_client_as_user(user).await;
    let tx = client.begin().await.unwrap();

    let (_, bot_account) = insert_test_bot_account(&tx, |_| ()).await.unwrap();

    tx.commit().await.unwrap();

    bot_account
}

async fn user_cannot_update_unowned_bot_account(client: &mut db::Client, bot_account_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let error = update_bot_account_by_id(
        &tx,
        bot_account_id,
        &BotAccountUpdate {
            description: Some("foo".to_nonempty_string()),
            is_staff: false,
        },
    )
    .await
    .unwrap_err();

    assert_matches!(error, DbError::ResourceNotFound);
}

async fn user_cannot_grant_access_to_unowned_bot_account(
    client: &mut db::Client,
    bot_account_id: Uuid,
) {
    let tx = client.begin().await.unwrap();

    let error = insert_bot_account_accesses(&tx, bot_account_id, &[Uuid::new_v4()])
        .await
        .unwrap_err();

    assert_matches!(error, DbError::PermissionDenied { .. });
}

async fn user_can_only_see_accessible_bot_accounts(
    client: &mut db::Client,
    accessible_bot_accounts: &[BotAccount],
) {
    let tx = client.begin().await.unwrap();

    let bot_accounts = select_bot_accounts(&tx, &BotAccountQuery::default())
        .await
        .unwrap();

    assert_eq!(bot_accounts, accessible_bot_accounts);
}

async fn user_cannot_see_inaccessible_bot_accounts(client: &mut db::Client, bot_account_id: Uuid) {
    let tx = client.begin().await.unwrap();

    let inaccessible = select_bot_accounts(
        &tx,
        &BotAccountPredicate::Id(UuidOperator::Eq(bot_account_id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(inaccessible, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_bot_accounts() {
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
    let user1_svc_acct = create_bot_account_for(user1_id).await;

    let user2_id = user2.record.id;
    let user2_svc_acct = create_bot_account_for(user2_id).await;

    let mut user1_client = db_client_as_user(user1_id).await;

    user_cannot_update_unowned_bot_account(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_grant_access_to_unowned_bot_account(&mut user1_client, user2_svc_acct.id).await;
    user_cannot_see_inaccessible_bot_accounts(&mut user1_client, user2_svc_acct.id).await;
    user_can_only_see_accessible_bot_accounts(
        &mut user1_client,
        std::slice::from_ref(&user1_svc_acct),
    )
    .await;

    // Now user1 grants access to user2
    let tx = user1_client.begin().await.unwrap();
    insert_bot_account_accesses(&tx, user1_svc_acct.id, &[user2_id])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut user2_client = db_client_as_user(user2_id).await;
    user_can_only_see_accessible_bot_accounts(&mut user2_client, &[user2_svc_acct, user1_svc_acct])
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn bot_account_owner_cannot_make_bot_account_staff() {
    let owner_id = create_test_user().await;
    let bot_account = create_bot_account_for(owner_id).await;

    let mut client = db_client_as_user(owner_id).await;

    // The owner may update the bot_account's principal, so the failure below
    // comes from the staff check
    let tx = client.begin().await.unwrap();
    set_is_staff(&tx, bot_account.id, false).await.unwrap();
    drop(tx);

    let tx = client.begin().await.unwrap();

    assert_eq!(
        set_is_staff(&tx, bot_account.id, true).await.unwrap_err(),
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"principal\""
                .to_owned()
        }
    );
}

// Owning a bot_account lets a user grant it the permissions they hold, even
// though they can't update people. That doesn't extend to bot_accounts they
// don't own
#[tokio::test(flavor = "multi_thread")]
async fn bot_account_owner_can_only_grant_permissions_to_owned_bot_account() {
    let permissions = may(Resource::Institution, &[Action::Create]);

    let owner_id = create_test_user_with(false, permissions.clone()).await;
    let owned_bot_account = create_bot_account_for(owner_id).await;
    let unowned_bot_account = create_bot_account_for(create_test_user().await).await;

    let mut client = db_client_as_user(owner_id).await;

    // This needs its own transaction because the failure aborts it
    let tx = client.begin().await.unwrap();
    let error = grant_permissions(&tx, unowned_bot_account.id, &permissions)
        .await
        .unwrap_err();

    assert_matches!(error, DbError::PermissionDenied { .. });
    drop(tx);

    let tx = client.begin().await.unwrap();
    grant_permissions(&tx, owned_bot_account.id, &permissions)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Using the permission would need the bot_account to be staff, so check
    // that it holds it
    let tx = client.begin().await.unwrap();
    let bot_account = tx
        .select_one(
            BotAccountPredicate::Id(UuidOperator::Eq(owned_bot_account.id)),
            select_bot_accounts_detailed,
        )
        .await
        .unwrap();

    assert_eq!(bot_account.permissions, permissions);
}

// `update` on people doesn't let a user change the permissions of a bot_account
// they don't own
#[tokio::test(flavor = "multi_thread")]
async fn person_updater_cannot_change_permissions_of_unowned_bot_account() {
    let create_institution = may(Resource::Institution, &[Action::Create]);
    let update_institution = may(Resource::Institution, &[Action::Update]);

    let owner_id = create_test_user_with(false, create_institution.clone()).await;
    let bot_account = create_bot_account_for(owner_id).await;

    let mut client = db_client_as_user(owner_id).await;
    let tx = client.begin().await.unwrap();
    grant_permissions(&tx, bot_account.id, &create_institution)
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
        grant_permissions(&tx, bot_account.id, &update_institution)
            .await
            .unwrap_err(),
        DbError::PermissionDenied { .. }
    );
    drop(tx);

    let tx = client.begin().await.unwrap();
    assert_eq!(
        revoke_permissions(&tx, bot_account.id, &create_institution)
            .await
            .unwrap_err(),
        DbError::ResourceNotFound
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn bot_account_cannot_change_itself() {
    let bot_account = create_bot_account_for(create_test_user().await).await;

    let mut client = db_client_as_user(bot_account.id).await;

    user_cannot_update_unowned_bot_account(&mut client, bot_account.id).await;
    user_cannot_grant_access_to_unowned_bot_account(&mut client, bot_account.id).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn bot_account_owner_can_update_staff_bot_account_without_changing_staff_status() {
    let owner_id = create_test_user().await;
    let bot_account = create_bot_account_for(owner_id).await;

    let mut admin = db_client_as_admin().await;
    let tx = admin.begin().await.unwrap();
    set_is_staff(&tx, bot_account.id, true).await.unwrap();
    tx.commit().await.unwrap();

    let mut client = db_client_as_user(owner_id).await;
    let tx = client.begin().await.unwrap();

    set_is_staff(&tx, bot_account.id, true).await.unwrap();
}
