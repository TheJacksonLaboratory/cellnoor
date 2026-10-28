use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::IdParam,
    state::AppState,
};

pub async fn remove_people_from_bot_account(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: bot_account_id }): Path<IdParam>,
    crate::extract::JsonExtractor(people): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            delete_bot_account_accesses(tx, bot_account_id, &people).await
        })
        .await
}

pub(in super::super::super) async fn delete_bot_account_accesses(
    tx: &db::Transaction<'_>,
    bot_account_id: Uuid,
    people: &[Uuid],
) -> Result<(), DbError> {
    static DELETE_SERVICE_ACCESSES: &str =
        "delete from bot_account_access where bot_account_id = $1 and person_id = any($2)";

    let n_rows = tx
        .execute(&Sql::new(
            DELETE_SERVICE_ACCESSES,
            vec![&bot_account_id, &people],
        ))
        .await?;

    if n_rows == 0 {
        return Err(DbError::ResourceNotFound);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use cellnoor_types::{bot_account::BotAccountPredicate, operator::UuidOperator};
    use uuid::Uuid;

    use crate::{
        handlers::bot_accounts::{
            access::remove_people::delete_bot_account_accesses,
            create::test::insert_test_bot_account, index_detailed::select_bot_accounts_detailed,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_people() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, bot_account) = insert_test_bot_account(&tx, |s| s.users = vec![Uuid::nil()])
            .await
            .unwrap();

        delete_bot_account_accesses(&tx, bot_account.id, &[Uuid::nil()])
            .await
            .unwrap();

        let selected = tx
            .select_one(
                BotAccountPredicate::Id(UuidOperator::Eq(bot_account.id)),
                select_bot_accounts_detailed,
            )
            .await
            .unwrap();

        assert!(selected.members.is_empty());
    }
}
