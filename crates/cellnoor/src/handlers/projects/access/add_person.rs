use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{JunctionTable, insert_many_to_many},
    },
    error::Error,
    handlers::path::IdParam,
    state::AppState,
};

pub async fn add_people_to_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: project_id }): Path<IdParam>,
    Json(people): Json<Vec<Uuid>>,
) -> Result<(), Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    insert_project_accesses(&tx, project_id, &people).await?;

    tx.commit().await?;

    Ok(())
}

pub async fn insert_project_accesses(
    tx: &db::Transaction<'_>,
    project_id: Uuid,
    people: &[Uuid],
) -> Result<(), Error> {
    insert_many_to_many(
        &tx,
        JunctionTable::ProjectAccess,
        ("project_id", project_id),
        ("person_id", &people),
    )
    .await?;

    Ok(())
}
