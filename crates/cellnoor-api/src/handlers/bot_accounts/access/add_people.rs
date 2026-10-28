use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::Relation;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::IdParam,
    state::AppState,
};

pub async fn add_people_to_bot_account(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: bot_account_id }): Path<IdParam>,
    crate::extract::JsonExtractor(people): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            insert_bot_account_accesses(tx, bot_account_id, &people).await
        })
        .await
}

pub(in super::super::super) async fn insert_bot_account_accesses(
    tx: &db::Transaction<'_>,
    bot_account_id: Uuid,
    people: &[Uuid],
) -> Result<(), DbError> {
    let accesses: Vec<_> = people
        .iter()
        .map(|&person_id| NewBotAccountAccess {
            bot_account_id,
            person_id,
        })
        .collect();

    tx.insert_many_on_conflict_do_nothing(&accesses).await
}

struct NewBotAccountAccess {
    bot_account_id: Uuid,
    person_id: Uuid,
}

impl Relation for NewBotAccountAccess {
    const NAME: &'static str = "bot_account_access";
}

impl Insert for NewBotAccountAccess {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            bot_account_id,
            person_id,
        } = self;

        vec![("bot_account_id", bot_account_id), ("person_id", person_id)]
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use crate::{
        handlers::{
            bot_accounts::{
                access::add_people::insert_bot_account_accesses,
                create::test::insert_test_bot_account,
            },
            people::create::insert_test_person_and_institution,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn add_people() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, bot_account) = insert_test_bot_account(&tx, |_| ()).await.unwrap();
        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        insert_bot_account_accesses(&tx, bot_account.id, &[person.record.id])
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn add_existing_member() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, bot_account) = insert_test_bot_account(&tx, |s| s.users = vec![Uuid::nil()])
            .await
            .unwrap();

        insert_bot_account_accesses(&tx, bot_account.id, &[Uuid::nil()])
            .await
            .unwrap();
    }
}
