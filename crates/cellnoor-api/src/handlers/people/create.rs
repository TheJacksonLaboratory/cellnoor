use std::sync::LazyLock;

use axum::{Json, extract::State};
use cellnoor_types::person::{AuthAccount, NewPerson, Person, PersonField, PersonSimpleFields};
use regex::Regex;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::{
        accounts::create::NewAccountRecord,
        people::{PersonError, show::select_person_by_id},
        permissions::grant_permissions,
        set_is_staff,
    },
    state::AppState,
};

pub async fn create_person(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(person): crate::extract::JsonExtractor<NewPerson>,
) -> Result<Json<Person>, PersonError> {
    state
        .in_transaction(user, async |tx| insert_person(tx, &person).await)
        .await
}

async fn insert_person(tx: &db::Transaction<'_>, new: &NewPerson) -> Result<Person, PersonError> {
    let NewPerson {
        simple,
        auth_account,
        permissions_to_grant,
    } = new;

    if let Some(email) = &simple.email {
        validate_email(email.as_ref())?;
    }

    let id = tx.insert_returning_id(&new.simple).await?;

    tokio::try_join!(
        set_is_staff(tx, id, simple.is_staff),
        grant_permissions(tx, id, permissions_to_grant)
    )?;

    let person = match auth_account.as_ref() {
        None => select_person_by_id(tx, id).await?,
        // These don't depend on one another
        Some(acc) => tokio::try_join!(select_person_by_id(tx, id), insert_account(tx, id, acc))?.0,
    };

    Ok(person)
}

async fn insert_account(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    account: &AuthAccount,
) -> Result<(), DbError> {
    match account {
        AuthAccount::Microsoft {
            microsoft_entra_oid,
        } => {
            tx.insert(&NewAccountRecord {
                person_id: user_id,
                auth_provider: account.as_ref(),
                auth_provider_user_id: microsoft_entra_oid.to_string(),
            })
            .await?
        }
    };

    Ok(())
}

impl Insert for PersonSimpleFields {
    type Field = PersonField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            name,
            email,
            institution_id,
            is_staff: _,
            orcid,
        } = self;

        vec![
            (PersonField::Name, name),
            (PersonField::Email, email),
            (PersonField::InstitutionId, institution_id),
            (PersonField::Orcid, orcid),
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

#[cfg(any(test, feature = "dev"))]
pub async fn insert_test_person_and_institution<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewPerson, Person), PersonError>
where
    F: FnMut(&mut NewPerson),
{
    use crate::{
        handlers::institutions::create::insert_test_institution, state::dev_util::ToNonemptyString,
    };

    let (_, institution) = insert_test_institution(tx, |_| ()).await?;

    let name = crate::db::dummy_data::random_name_for("person");
    let email = format!("{name}@example.com").to_nonempty_string();
    let mut new = NewPerson {
        simple: PersonSimpleFields {
            name,
            email: Some(email),
            institution_id: *institution.record.id,
            is_staff: false,
            orcid: None,
        },
        auth_account: Some(AuthAccount::Microsoft {
            microsoft_entra_oid: Uuid::new_v4(),
        }),
        permissions_to_grant: Vec::new(),
    };

    modify(&mut new);

    let inserted = insert_person(tx, &new).await?;
    Ok((new, inserted))
}

#[cfg(test)]
pub mod test {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::insert_test_person_and_institution;
    use crate::{
        db::DbError,
        handlers::people::{PersonError, create::validate_email},
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
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

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_email() {
        assert_eq!(
            validate_email("email").unwrap_err(),
            PersonError::InvalidEmail {
                email: "email".to_owned()
            }
        );
    }
}
