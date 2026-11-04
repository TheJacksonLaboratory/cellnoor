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
    static SELECT_ACCOUNT: &str = "select person_account from person_account where id = $1 and \
                                   auth_provider = $2 and auth_provider_user_id = $3";

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

    tx.insert(&record).await?;

    let NewAccountRecord {
        person_id,
        auth_provider,
        auth_provider_user_id,
    } = &record;

    tx.query_one_into(&Sql::new(
        SELECT_ACCOUNT,
        vec![person_id, auth_provider, auth_provider_user_id],
    ))
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
        account::{NewAccount, PersonAccount},
        person::{Account, Person, SavedPersonRecord},
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

        let email = format!("{}@example.com", Uuid::new_v4());
        let (
            _,
            Person {
                record: SavedPersonRecord { id, name, .. },
                links: _,
            },
        ) = insert_test_person_and_institution(&tx, |p| {
            p.account = Account::None {
                email: email.to_nonempty_string(),
            }
        })
        .await
        .unwrap();

        let microsoft_entra_oid = Uuid::new_v4();
        let account = insert_account(
            &tx,
            &NewAccount::Microsoft {
                person_id: id,
                microsoft_entra_oid,
            },
        )
        .await
        .unwrap();

        assert_eq!(
            account,
            PersonAccount {
                id,
                name,
                email: Some(email.to_nonempty_string()),
                auth_provider: "microsoft".to_owned(),
                auth_provider_user_id: microsoft_entra_oid.to_string(),
            }
        );
    }
}
