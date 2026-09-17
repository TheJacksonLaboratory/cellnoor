use axum::{Json, extract::State};
use cellnoor_types::nonempty::NonemptyString;
use postgres_types::FromSql;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    state::AppState,
};

// We don't define this in cellnoor-types because it's not really a public
// endpoint
#[derive(Clone, Debug, Serialize, JsonSchema, FromSql)]
#[postgres(name = "person_account")]
pub struct PersonAccount {
    id: Uuid,
    name: NonemptyString,
    email: Option<NonemptyString>,
    auth_provider: String,
    auth_provider_user_id: String,
}

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
            accounts::index::select_accounts,
            people::create::test::insert_test_person_and_institution,
        },
        state::test_util::db_client_as_admin,
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
