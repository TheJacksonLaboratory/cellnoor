use axum::{Json, extract::State};
use cellnoor_types::account::PersonAccount;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    state::AppState,
};

pub async fn index_accounts(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<PersonAccount>>, DbError> {
    state
        .in_transaction(user, async |tx| select_accounts(tx).await)
        .await
}

async fn select_accounts(tx: &db::Transaction<'_>) -> Result<Vec<PersonAccount>, DbError> {
    static SELECT_API_KEYS: &str = include_str!("index/select.sql");

    let sql = Sql::new(SELECT_API_KEYS, vec![]);

    tx.query_into(&sql).await
}

#[cfg(test)]
mod tests {
    use crate::{
        handlers::{
            accounts::index::select_accounts, people::create::insert_test_person_and_institution,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test]
    async fn select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        select_accounts(&tx).await.unwrap();
    }
}
