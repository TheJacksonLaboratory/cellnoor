use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::person::{Person, PersonField, PersonUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    handlers::{
        IdParam,
        people::{
            PersonError,
            create::{person_field_value_pairs, validate_email},
            show::select_person_by_id,
        },
        set_is_staff,
    },
    state::AppState,
};

pub async fn update_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(person): crate::extract::JsonExtractor<PersonUpdate>,
) -> Result<Json<Person>, PersonError> {
    state
        .in_transaction(user, async |tx| update_person_by_id(tx, id, &person).await)
        .await
}

pub(in super::super) async fn update_person_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &PersonUpdate,
) -> Result<Person, PersonError> {
    let PersonUpdate { simple, email } = update;
    if let Some(email) = email {
        validate_email(email.as_ref())?;
    }

    tx.update(id, update).await?;
    set_is_staff(tx, id, simple.is_staff).await?;

    Ok(select_person_by_id(tx, id).await?)
}

impl Insert for PersonUpdate {
    type Field = PersonField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self { simple, email } = self;

        person_field_value_pairs(simple, email)
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::person::{Person, PersonUpdate, SavedPersonRecord};

    use crate::{
        handlers::people::{
            create::insert_test_person_and_institution, update::update_person_by_id,
        },
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            mut pre_update,
            Person {
                record: SavedPersonRecord { id, .. },
                links: _,
            },
        ) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        pre_update.simple.name = "updated".to_nonempty_string();
        let update_to_apply = PersonUpdate {
            simple: pre_update.simple,
            email: Some("something@example.com".to_nonempty_string()),
        };

        update_person_by_id(&tx, id, &update_to_apply)
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_without_email() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (pre_update, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        let update_to_apply = PersonUpdate {
            simple: pre_update.simple,
            email: None,
        };

        update_person_by_id(&tx, person.record.id, &update_to_apply)
            .await
            .unwrap();
    }
}
