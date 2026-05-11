use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::person::{NewPerson, Person};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToUpdateClause},
    },
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        people::{
            create::{
                create_db_user, grant_permissions_to_db_user, revoke_permissions_from_db_user,
                validate_email,
            },
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
        name,
        institution_id,
        email,
        orcid,
        is_staff,
        grant_permissions: permissions_to_grant,
        revoke_permissions: permissions_to_revoke,
    }: &NewPerson,
) -> Result<Person, ErrorInner> {
    validate_email(email.as_ref())?;

    let fields: FieldValuePairs<_> = [
        ("name", name),
        ("institution_id", institution_id),
        ("email", email),
        ("orcid", orcid),
    ];

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update person set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    let user_operations = async || {
        create_db_user(tx, id, *is_staff).await?;
        grant_permissions_to_db_user(tx, id, permissions_to_grant).await?;
        revoke_permissions_from_db_user(tx, id, permissions_to_revoke).await?;

        Ok(())
    };

    let (_, person) = tokio::try_join!(user_operations(), select_person_by_id(tx, id))?;

    Ok(person)
}

#[cfg(test)]
mod test {
    use cellnoor_types::person::NewPerson;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::{Error, ErrorInner},
        handlers::people::{
            create::{insert_person, test::new_person},
            update::update_person_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let to_insert = new_person();
        let person = insert_person(&tx, &to_insert).await.unwrap();

        let new_data = NewPerson {
            name: "updated".to_nonempty_string(),
            ..to_insert
        };

        let updated = update_person_by_id(&tx, person.record.id, &new_data)
            .await
            .unwrap();

        assert_eq!(updated.record.id, person.record.id);
        assert_eq!(updated.record.name, new_data.name);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = update_person_by_id(&tx, Uuid::new_v4(), &new_person())
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
