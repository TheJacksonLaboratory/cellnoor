use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, person::Person};
use cellnoor_schema::people;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn show_person(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Person>, db::Error> {
    select_person_by_id(id, &mut db_conn).await.map(Json)
}

pub async fn select_person_by_id(
    person_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<Person, db::Error> {
    Ok(Person::query()
        .filter(people::id.eq(person_id))
        .first(db_conn)
        .await?)
}
