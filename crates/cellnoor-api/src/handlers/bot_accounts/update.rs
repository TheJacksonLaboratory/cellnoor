use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::bot_account::{BotAccount, BotAccountUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{IdParam, bot_accounts::index::select_bot_account_by_id, set_is_staff},
    state::AppState,
};

pub async fn update_bot_account(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(bot_account): crate::extract::JsonExtractor<BotAccountUpdate>,
) -> Result<Json<BotAccount>, DbError> {
    state
        .in_transaction(user, async |tx| {
            update_bot_account_by_id(tx, id, &bot_account).await
        })
        .await
}

pub(in super::super) async fn update_bot_account_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &BotAccountUpdate,
) -> Result<BotAccount, DbError> {
    tx.update(id, update).await?;
    set_is_staff(tx, id, update.is_staff).await?;

    select_bot_account_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::bot_account::BotAccountUpdate;

    use crate::{
        handlers::bot_accounts::{
            create::test::insert_test_bot_account, update::update_bot_account_by_id,
        },
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_bot_account(&tx, |_| ()).await.unwrap();

        let update = BotAccountUpdate {
            description: Some("updated".to_nonempty_string()),
            is_staff: false,
        };

        update_bot_account_by_id(&tx, inserted.id, &update)
            .await
            .unwrap();
    }
}
