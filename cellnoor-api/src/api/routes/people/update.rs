use crate::api::routes::people::create::validate_email;
use crate::api::routes::people::show::select_person_by_id;
use crate::db::DbConnection;
use crate::{db, state::AppState};
use aide::OperationIo;
use axum::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::IdParameter;
use cellnoor_models::person::Person;
use cellnoor_models::person::{NewPerson, PersonUpdate};
use cellnoor_schema::people;
use diesel::{
    prelude::*,
    sql_types::{Array, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;
use uuid::Uuid;

pub async fn update_person(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(mut person_update): Json<PersonUpdate>,
) -> Result<Json<Person>, super::create::Error> {
    if let Some(email) = person_update.email() {
        validate_email(email)?;
    }

    person_update.set_id(id);

    update_person_inner(person_update, &mut db_conn).await?;

    Ok(select_person_by_id(id, &mut db_conn).await.map(Json)?)
}

async fn update_person_inner(
    update: PersonUpdate,
    db_conn: &mut DbConnection,
) -> Result<(), super::create::Error> {
    diesel::update(people::table)
        .set(update)
        .execute(db_conn)
        .await?;

    Ok(())
}
