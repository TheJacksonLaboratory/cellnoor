use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::IdParameter;
use cellnoor_schema::project_people;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn remove_person_from_project(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id: project_id }): Path<IdParameter>,
    Json(IdParameter { id: person_id }): Json<IdParameter>,
) -> Result<(), db::Error> {
    delete_project_person_mapping(project_id, person_id, &mut db_conn).await
}

async fn delete_project_person_mapping(
    _project_id: Uuid,
    person_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<(), db::Error> {
    diesel::delete(project_people::table.filter(project_people::person_id.eq(person_id)))
        .execute(db_conn)
        .await?;

    Ok(())
}
