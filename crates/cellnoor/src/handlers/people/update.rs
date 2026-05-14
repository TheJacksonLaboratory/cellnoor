use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::person::{NewPerson, NewPersonRecord, Person};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, FieldValuePairs, ToUpdateClause},
    },
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        people::{
            create::{provision_db_user, validate_email},
            show::select_person_by_id,
        },
    },
    state::AppState,
};

pub async fn update_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(person): Json<NewPerson>,
) -> Result<Json<Person>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_person_by_id(&tx, id, &person).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_person_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    NewPerson {
        record,
        is_staff,
        permissions_to_grant,
        permissions_to_revoke,
    }: &NewPerson,
) -> Result<Person, ErrorInner> {
    validate_email(record.email.as_ref().map(NonemptyString::as_ref))?;

    let fields = record.as_field_value_pairs();

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update person set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    let (_, person) = tokio::try_join!(
        provision_db_user(
            tx,
            id,
            *is_staff,
            permissions_to_grant,
            permissions_to_revoke
        ),
        select_person_by_id(tx, id)
    )?;

    Ok(person)
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        person::{NewPerson, NewPersonRecord, Person, SavedPersonRecord},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
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
        ) = insert_test_person_and_institution(&tx, identity).await;
        pre_update.record.name = "updated".to_nonempty_string();

        let Person {
            record: post_update_record,
            links: _,
        } = update_person_by_id(&tx, *id, &pre_update).await.unwrap();

        let expected_record = SavedPersonRecord {
            id,
            name: pre_update.record.name,
            institution_id: pre_update.record.institution_id,
            email: pre_update.record.email,
            orcid: pre_update.record.orcid,
        };

        assert_eq!(post_update_record, expected_record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let new_data = NewPerson {
            record: NewPersonRecord {
                id: NoId {},
                name: "missing".to_nonempty_string(),
                institution_id: Uuid::new_v4(),
                email: Some(format!("{}@jax.org", Uuid::new_v4()).to_nonempty_string()),
                orcid: None,
            },
            is_staff: false,
            permissions_to_grant: vec![].into(),
            permissions_to_revoke: vec![].into(),
        };

        let error = update_person_by_id(&tx, Uuid::new_v4(), &new_data)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
