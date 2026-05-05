use axum::{Json, extract::State};
use cellnoor_types::{
    institution::{Institution, NewInstitution},
    person::{NewPerson, Person, ResourcePermission},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser, db, error::Error, handlers::people::show::select_person_by_id, state::AppState,
};

pub async fn create_person(
    State(state): State<AppState>,
    user: AuthUser,
    Json(person): Json<NewPerson>,
) -> Result<Json<Person>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_person(&tx, &person).await.map(Json);

    tx.commit().await?;

    response
}

pub(super) async fn insert_person(
    tx: &db::Transaction<'_>,
    NewPerson {
        name,
        institution_id,
        email,
        orcid,
        is_staff,
        grant_permissions: permissions,
        revoke_permissions: _, // we ignore `revoke_permissions` on a creation
    }: &NewPerson,
) -> Result<Person, crate::error::Error> {
    // Simple queries can be written inline
    let person_id = tx
        .query_one_into(
            "insert into person (name, institution_id, email, orcid) values ($1, $2, $3) \
             returning id",
            &[name, institution_id, email, orcid],
        )
        .await?;

    tokio::try_join!(
        create_user(tx, person_id, *is_staff),
        grant_permissions_to_user(tx, person_id, permissions)
    )?;

    let person = select_person_by_id(tx, person_id).await?;

    Ok(person)
}

pub(super) async fn create_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    is_staff: bool,
) -> Result<(), Error> {
    tx.execute(
        "select create_person_user_if_not_exists($1, $2)",
        &[&user_id, &is_staff],
    )
    .await?;

    Ok(())
}

pub(super) async fn grant_permissions_to_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &[ResourcePermission],
) -> Result<(), Error> {
    let grant_stmts: Vec<_> = permissions
        .iter()
        .map(|p| construct_grant_statement(user_id, p))
        .collect();

    let grant_ops = grant_stmts.iter().map(|s| tx.execute(s, &[]));
    futures::future::try_join_all(grant_ops).await?;

    Ok(())
}

fn construct_grant_statement(user_id: Uuid, resource_permissions: &ResourcePermission) -> String {
    let resource_name = resource_permissions.as_ref();
    let actions = match resource_permissions {
        ResourcePermission::Institution(a)
        | ResourcePermission::Person(a)
        | ResourcePermission::Project(a)
        | ResourcePermission::Specimen(a)
        | ResourcePermission::ChromiumExperimentalEntities(a)
        | ResourcePermission::ChromiumDataset(a) => a,
    };

    let actions: Vec<_> = actions.iter().map(|a| a.as_ref()).collect();
    let actions = actions.join(", ");

    format!(r#"grant {actions} to "{user_id}" on {resource_name}"#)
}

#[cfg(test)]
pub mod test {}
