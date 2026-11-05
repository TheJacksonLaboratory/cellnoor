use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::person::{Person, PersonUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    handlers::{
        IdParam,
        people::{PersonError, create::validate_email, show::select_person_by_id},
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
    if let Some(email) = &update.email {
        validate_email(email.as_ref())?;
    }

    tokio::try_join!(tx.update(id, update), set_is_staff(tx, id, update.is_staff))?;

    Ok(select_person_by_id(tx, id).await?)
}

#[cfg(test)]
mod test {

    use cellnoor_types::person::{Person, SavedPersonRecord};

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
            pre_update,
            Person {
                record: SavedPersonRecord { id, .. },
                links: _,
            },
        ) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        let mut update_to_apply = pre_update.simple;

        update_to_apply.name = "updated".to_nonempty_string();
        update_to_apply.email = Some("something@example.com".to_nonempty_string());

        update_person_by_id(&tx, id, &update_to_apply)
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_remove_email() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (pre_update, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        let mut update_to_apply = pre_update.simple;
        update_to_apply.email = None;

        update_person_by_id(&tx, person.record.id, &update_to_apply)
            .await
            .unwrap();
    }
}
