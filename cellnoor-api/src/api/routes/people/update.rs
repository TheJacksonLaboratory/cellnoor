use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    person::{Person, PersonUpdate},
};
use cellnoor_schema::people;
use diesel_async::RunQueryDsl;

use crate::{
    api::routes::people::{create::validate_email, show::select_person_by_id},
    db::DbConnection,
    state::AppState,
};

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

pub async fn update_person_inner(
    update: PersonUpdate,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), super::create::Error> {
    diesel::update(people::table)
        .set(update)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}
