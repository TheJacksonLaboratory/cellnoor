use axum::{Json, extract::State};
use cellnoor_types::bot_account::{
    BotAccount, BotAccountField, BotAccountSimpleFields, NewBotAccount,
};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::{
        bot_accounts::{
            access::add_people::insert_bot_account_accesses, index::select_bot_account_by_id,
        },
        permissions::grant_permissions,
        set_is_staff,
    },
    state::AppState,
};

pub async fn create_bot_account(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(bot_account): crate::extract::JsonExtractor<NewBotAccount>,
) -> Result<Json<BotAccount>, DbError> {
    state
        .in_transaction(user, async |tx| insert_bot_account(tx, &bot_account).await)
        .await
}

async fn insert_bot_account(
    tx: &db::Transaction<'_>,
    NewBotAccount {
        record,
        users,
        permissions_to_grant,
    }: &NewBotAccount,
) -> Result<BotAccount, DbError> {
    let id = tx.insert_returning_id(record).await?;
    set_is_staff(tx, id, record.is_staff).await?;
    grant_permissions(tx, id, permissions_to_grant).await?;

    insert_bot_account_accesses(tx, id, users).await?;

    select_bot_account_by_id(tx, id).await
}

// `owned_by` is intentionally omitted: the database fills it from
// `app_user_id()`, and row-level security guarantees it equals the caller.
// `is_staff` is a column of `principal`, not `bot_account` (see `set_is_staff`)
impl Insert for BotAccountSimpleFields {
    type Field = BotAccountField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            description,
            is_staff: _,
        } = self;

        vec![(BotAccountField::Description, description)]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::bot_account::{BotAccount, BotAccountSimpleFields, NewBotAccount};
    use uuid::Uuid;

    use crate::{
        db::{self, DbError},
        handlers::bot_accounts::create::insert_bot_account,
        state::dev_util::db_client_as_admin,
    };

    pub async fn insert_test_bot_account<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewBotAccount, BotAccount), DbError>
    where
        F: FnMut(&mut NewBotAccount),
    {
        let mut new = NewBotAccount {
            record: BotAccountSimpleFields {
                description: Some(crate::db::dummy_data::random_name_for("bot_account")),
                is_staff: false,
            },
            users: vec![Uuid::nil()],
            permissions_to_grant: Vec::new(),
        };

        modify(&mut new);

        let inserted = insert_bot_account(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_bot_account(&tx, |_| ()).await.unwrap();
    }
}
