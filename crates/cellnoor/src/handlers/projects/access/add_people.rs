use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn add_people_to_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: project_id }): Path<IdParam>,
    Json(people): Json<Vec<Uuid>>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = insert_project_accesses(&tx, project_id, &people)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_project_accesses(
    tx: &db::Transaction<'_>,
    project_id: Uuid,
    people: &[Uuid],
) -> Result<(), ErrorInner> {
    let accesses: Vec<_> = people
        .iter()
        .map(|&person_id| NewProjectAccess {
            project_id,
            person_id,
        })
        .collect();

    futures::future::try_join_all(
        accesses
            .iter()
            .map(|a| db::insert_into_no_returning(tx, "project_access", a)),
    )
    .await?;

    Ok(())
}

struct NewProjectAccess {
    project_id: Uuid,
    person_id: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewProjectAccess {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            project_id,
            person_id,
        } = self;

        [("project_id", project_id), ("person_id", person_id)]
    }
}
