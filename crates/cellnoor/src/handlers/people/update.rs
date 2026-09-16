use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::person::{Person, PersonField, PersonUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        people::{
            create::{person_field_value_pairs, validate_email},
            show::select_person_by_id,
        },
        permissions::{grant_permissions, revoke_permissions},
        set_is_staff,
    },
    state::AppState,
};

pub async fn update_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(person): Json<PersonUpdate>,
) -> Result<Json<Person>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_person_by_id(&tx, id, &person).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_person_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &PersonUpdate,
) -> Result<Person, ErrorInner> {
    let PersonUpdate {
        simple,
        email,
        permissions_to_grant,
        permissions_to_revoke,
    } = update;
    validate_email(email.as_ref())?;

    db::update(tx, "person", id, update).await?;
    set_is_staff(tx, id, simple.is_staff).await?;
    grant_permissions(tx, id, permissions_to_grant).await?;
    revoke_permissions(tx, id, permissions_to_revoke).await?;

    select_person_by_id(tx, id).await
}

impl AsFieldValuePairs<PersonField, 4> for PersonUpdate {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, PersonField, 4> {
        let Self {
            simple,
            email,
            permissions_to_grant: _,
            permissions_to_revoke: _,
        } = self;

        person_field_value_pairs(simple, email)
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::person::{Person, PersonUpdate, SavedPersonRecord};

    use crate::{
        handlers::people::{
            create::test::insert_test_person_and_institution, update::update_person_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
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
            email: "something@example.com".to_nonempty_string(),
            permissions_to_grant: Vec::new(),
            permissions_to_revoke: Vec::new(),
        };

        update_person_by_id(&tx, id, &update_to_apply)
            .await
            .unwrap();
    }
}
