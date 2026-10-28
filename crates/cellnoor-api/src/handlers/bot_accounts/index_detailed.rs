use axum::{Json, extract::State};
use cellnoor_types::bot_account::{BotAccountDetailed, BotAccountQuery};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_bot_accounts_detailed(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<BotAccountQuery>,
) -> Result<Json<Vec<BotAccountDetailed>>, DbError> {
    state
        .in_transaction(user, async |tx| {
            select_bot_accounts_detailed(tx, &query).await
        })
        .await
}

pub(in super::super) async fn select_bot_accounts_detailed(
    tx: &db::Transaction<'_>,
    query: &BotAccountQuery,
) -> Result<Vec<BotAccountDetailed>, DbError> {
    static SELECT_SERVICES_DETAILED: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_detailed.sql"));

    tx.select(&SELECT_SERVICES_DETAILED, query).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        bot_account::{BotAccountField, BotAccountPredicate, BotAccountQuery},
        operator::UuidOperator,
        permission::{Action, Permission, Resource},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::bot_accounts::{
            create::test::insert_test_bot_account, index_detailed::select_bot_accounts_detailed,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let permission = Permission {
            resource: Resource::Specimen,
            action: Action::Create,
        };
        let (_, inserted) = insert_test_bot_account(&tx, |s| {
            s.users = vec![Uuid::nil()];
            s.permissions_to_grant = vec![permission];
        })
        .await
        .unwrap();

        let query =
            BotAccountQuery::from_filter(BotAccountPredicate::Id(UuidOperator::Eq(inserted.id)));
        let selected = select_bot_accounts_detailed(&tx, &query).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].bot_account, inserted);
        assert_eq!(selected[0].permissions, [permission]);
        assert_eq!(selected[0].members, [Uuid::nil()]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<BotAccountField>(&tx, "bot_account_detailed").await;
    }
}
