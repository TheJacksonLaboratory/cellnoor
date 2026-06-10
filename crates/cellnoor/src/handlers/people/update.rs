use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::person::{PermissionsToGrant, PermissionsToRevoke, Person, PersonUpdate};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        people::{
            create::{permission_to_permission_set, validate_email},
            show::select_person_by_id,
        },
    },
    state::AppState,
};

pub async fn update_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(mut person): Json<PersonUpdate>,
) -> Result<Json<Person>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_person_by_id(&tx, id, &mut person).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_person_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    PersonUpdate {
        simple,
        email,
        permissions_to_grant,
        permissions_to_revoke,
    }: &PersonUpdate,
) -> Result<Person, ErrorInner> {
    validate_email(email.as_ref())?;

    db::update(tx, "person", id, simple).await?;

    let no_grants = PermissionsToGrant::default();
    let no_revocations = PermissionsToRevoke::default();

    let permissions_to_grant = permissions_to_grant.as_ref().unwrap_or(&no_grants);
    let permissions_to_revoke = permissions_to_revoke.as_ref().unwrap_or(&no_revocations);

    let (_, person) = tokio::try_join!(
        update_permissions(tx, id, &permissions_to_grant, &permissions_to_revoke),
        select_person_by_id(tx, id)
    )?;

    Ok(person)
}

async fn update_permissions(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions_to_grant: &PermissionsToGrant,
    permissions_to_revoke: &PermissionsToRevoke,
) -> Result<(), ErrorInner> {
    grant_permissions(tx, user_id, permissions_to_grant).await?;
    revoke_permissions(tx, user_id, permissions_to_revoke).await?;

    Ok(())
}

async fn grant_permissions(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions_to_grant: &PermissionsToGrant,
) -> Result<(), ErrorInner> {
    let permissions_to_grant: Vec<_> = permissions_to_grant
        .iter()
        .map(permission_to_permission_set)
        .collect();

    tx.execute_raw_sql(
        "select grant_permissions_to_person($1, $2)",
        &[&user_id, &permissions_to_grant],
    )
    .await?;

    Ok(())
}

async fn revoke_permissions(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions_to_revoke: &PermissionsToRevoke,
) -> Result<(), ErrorInner> {
    let permissions_to_revoke: Vec<_> = permissions_to_revoke
        .iter()
        .map(permission_to_permission_set)
        .collect();

    tx.execute_raw_sql(
        "select revoke_permissions_from_person($1, $2)",
        &[&user_id, &permissions_to_revoke],
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {

    use cellnoor_types::person::{NewPerson, Person, PersonUpdate, SavedPersonRecord};

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
            pre_update,
            Person {
                record: SavedPersonRecord { id, .. },
                links: _,
            },
        ) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        let NewPerson::Microsoft { mut common, .. } = pre_update else {
            unreachable!()
        };

        common.simple.name = "updated".to_nonempty_string();
        let update_to_apply = PersonUpdate {
            simple: common.simple,
            email: "something@example.com".to_nonempty_string(),
            permissions_to_grant: None,
            permissions_to_revoke: None,
        };

        update_person_by_id(&tx, id, &update_to_apply)
            .await
            .unwrap();
    }
}
