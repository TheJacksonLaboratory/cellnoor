use axum::{Json, extract::State};
use cellnoor_types::{
    bot_account::{BotAccount, BotAccountPredicate, BotAccountQuery},
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_bot_accounts(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<BotAccountQuery>,
) -> Result<Json<Vec<BotAccount>>, DbError> {
    state
        .in_transaction(user, async |tx| select_bot_accounts(tx, &query).await)
        .await
}

pub(in super::super) async fn select_bot_accounts(
    tx: &db::Transaction<'_>,
    query: &BotAccountQuery,
) -> Result<Vec<BotAccount>, DbError> {
    static SELECT_SERVICES: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    tx.select(&SELECT_SERVICES, query).await
}

// Even though there's no bot-accounts/{id} route, this function is
// still helpful
pub(super) async fn select_bot_account_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<BotAccount, DbError> {
    tx.select_one(
        BotAccountPredicate::Id(UuidOperator::Eq(id)),
        select_bot_accounts,
    )
    .await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        bot_account::{BotAccountField, BotAccountPredicate, BotAccountQuery},
        operator::UuidOperator,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::bot_accounts::{
            create::test::insert_test_bot_account, index::select_bot_accounts,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_bot_account(&tx, |_| ()).await.unwrap();

        let query =
            BotAccountQuery::from_filter(BotAccountPredicate::Id(UuidOperator::Eq(inserted.id)));
        let selected = select_bot_accounts(&tx, &query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, inserted.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<BotAccountField>(&tx, "bot_account_public").await;
    }
}
