use axum::{Json, extract::State};
use cellnoor_types::{
    Relation,
    account::{NewAccount, PersonAccount},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert, Sql},
    state::AppState,
};

pub async fn create_account(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(account): crate::extract::JsonExtractor<NewAccount>,
) -> Result<Json<PersonAccount>, DbError> {
    state
        .in_transaction(user, async |tx| insert_account(tx, &account).await)
        .await
}

pub(in super::super) async fn insert_account(
    tx: &db::Transaction<'_>,
    new: &NewAccount,
) -> Result<PersonAccount, DbError> {
    static SELECT_ACCOUNT: &str = "select person_account from person_account where id = $1";

    let record = match new {
        NewAccount::Microsoft {
            person_id,
            microsoft_entra_oid,
        } => NewAccountRecord {
            person_id: *person_id,
            auth_provider: new.as_ref(),
            auth_provider_user_id: microsoft_entra_oid.to_string(),
        },
    };

    let id = tx.insert_returning_id(&record).await?;

    tx.query_one_into(&Sql::new(SELECT_ACCOUNT, vec![&id]))
        .await
}

pub(in super::super) struct NewAccountRecord<'a> {
    pub person_id: Uuid,
    pub auth_provider: &'a str,
    pub auth_provider_user_id: String,
}

impl Relation for NewAccountRecord<'_> {
    const NAME: &'static str = "account";
}

impl Insert for NewAccountRecord<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            person_id,
            auth_provider,
            auth_provider_user_id,
        } = self;

        vec![
            ("person_id", person_id),
            ("auth_provider", auth_provider),
            ("auth_provider_user_id", auth_provider_user_id),
        ]
    }
}

#[cfg(test)]
mod tests {
    use cellnoor_types::{
        account::NewAccount,
        person::{Person, SavedPersonRecord},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        handlers::{
            accounts::create::insert_account, people::create::insert_test_person_and_institution,
        },
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_for_person_with_email() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            _,
            Person {
                record:
                    SavedPersonRecord {
                        id: person_id,
                        name,
                        ..
                    },
                links: _,
            },
        ) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        let microsoft_entra_oid = Uuid::new_v4();
        let account = insert_account(
            &tx,
            &NewAccount::Microsoft {
                person_id,
                microsoft_entra_oid,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            (
                account.person_id,
                account.name,
                account.auth_provider,
                account.auth_provider_user_id
            ),
            (
                person_id,
                name.to_nonempty_string(),
                "microsoft".to_owned(),
                microsoft_entra_oid.to_string()
            )
        );
    }
}
