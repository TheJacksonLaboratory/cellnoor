use std::sync::LazyLock;

use axum::{Json, extract::State};
use cellnoor_types::{
    Relation,
    nonempty::NonemptyString,
    person::{Account, NewPerson, Person, PersonField, PersonSimpleFields},
};
use postgres_types::ToSql;
use regex::Regex;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::{
        people::{PersonError, show::select_person_by_id},
        permissions::grant_permissions,
        set_is_staff,
    },
    state::AppState,
};

pub async fn create_person(
    State(state): State<AppState>,
    user: AuthUser,
    Json(person): Json<NewPerson>,
) -> Result<Json<Person>, PersonError> {
    state
        .in_transaction(user, async |tx| insert_person(tx, &person).await)
        .await
}

async fn insert_person(
    tx: &db::Transaction<'_>,
    new: &NewPerson,
) -> Result<Person, PersonError> {
    let NewPerson {
        simple,
        account,
        permissions_to_grant,
    } = new;

    match account {
        Account::None { email } => validate_email(email.as_ref())?,
        Account::Microsoft {
            microsoft_entra_oid: _,
        } => (),
    };

    let id = tx.insert_returning_id(new).await?;
    set_is_staff(tx, id, simple.is_staff).await?;
    grant_permissions(tx, id, permissions_to_grant).await?;

    // These don't depend on one another
    let (person, ()) =
        tokio::try_join!(select_person_by_id(tx, id), insert_account(tx, id, account))?;

    Ok(person)
}

async fn insert_account(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    account: &Account,
) -> Result<(), DbError> {
    match account {
        Account::Microsoft {
            microsoft_entra_oid,
        } => {
            tx.insert(&NewAccountRecord::new(
                user_id,
                account.as_ref(),
                microsoft_entra_oid,
            ))
            .await?
        }
        Account::None { email: _ } => (),
    };

    Ok(())
}

// `is_staff` is a column of `principal`, not `person`, so it's set separately
// (see `set_is_staff`)
pub(super) fn person_field_value_pairs<'a>(
    simple: &'a PersonSimpleFields,
    email: &'a (dyn ToSql + Sync),
) -> FieldValues<'a, PersonField> {
    use PersonField::*;

    let PersonSimpleFields {
        name,
        institution_id,
        is_staff: _,
        orcid,
    } = simple;

    vec![
        (Name, name),
        (InstitutionId, institution_id),
        (Orcid, orcid),
        (Email, email),
    ]
}

impl Insert for NewPerson {
    type Field = PersonField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            simple,
            account,
            permissions_to_grant: _,
        } = self;

        let email: &(dyn ToSql + Sync) = match account {
            Account::Microsoft {
                microsoft_entra_oid: _,
            } => &None::<NonemptyString>,
            Account::None { email } => email,
        };

        person_field_value_pairs(simple, email)
    }
}

struct NewAccountRecord<'a> {
    person_id: Uuid,
    auth_provider: &'a str,
    auth_provider_user_id: String,
}

impl<'a> NewAccountRecord<'a> {
    fn new(
        person_id: Uuid,
        auth_provider: &'a str,
        auth_provider_user_id: impl std::fmt::Display,
    ) -> Self {
        Self {
            person_id,
            auth_provider,
            auth_provider_user_id: auth_provider_user_id.to_string(),
        }
    }
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

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

pub(super) fn validate_email(email: &str) -> Result<(), PersonError> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(PersonError::InvalidEmail {
            email: email.to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::person::{Account, NewPerson, Person, PersonSimpleFields};
    use pretty_assertions::{assert_eq, assert_str_eq};
    use uuid::Uuid;

    use crate::{
        db::{self, DbError},
        handlers::{
            institutions::create::test::insert_test_institution,
            people::{PersonError, create::insert_person},
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_person_and_institution<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewPerson, Person), PersonError>
    where
        F: FnMut(&mut NewPerson),
    {
        let (_, institution) = insert_test_institution(tx, |_| ()).await?;

        let mut new = NewPerson {
            simple: PersonSimpleFields {
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                institution_id: *institution.record.id,
                is_staff: false,
                orcid: None,
            },
            account: Account::Microsoft {
                microsoft_entra_oid: Uuid::new_v4(),
            },
            permissions_to_grant: Vec::new(),
        };

        modify(&mut new);

        let inserted = insert_person(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_email() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_person_and_institution(&tx, |p| {
            p.account = Account::None {
                email: "email@example.com".to_nonempty_string(),
            }
        })
        .await
        .unwrap();

        assert_str_eq!(inserted.record.email.unwrap().as_ref(), "email@example.com");
    }

    // This test just ensures that the correct error is returned when there's an
    // invalid foreign key
    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_reference_error() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = insert_test_person_and_institution(&tx, |p| {
            p.simple.institution_id = Uuid::new_v4();
        })
        .await
        .unwrap_err();

        assert_eq!(
            error,
            DbError::InvalidReference {
                referencing_resource: "person".to_owned(),
                referencing_field: "institution_id".to_owned(),
            }
            .into(),
        );
    }
}
