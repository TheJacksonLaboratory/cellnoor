use crate::api::routes::people::create::validate_email;
use crate::api::routes::people::show::select_person_by_id;
use crate::db::DbConnection;
use crate::state::AppState;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use cellnoor_models::IdParameter;
use cellnoor_models::person::Person;
use cellnoor_models::person::PersonUpdate;
use cellnoor_schema::people;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

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
    db_conn: &mut DbConnection,
) -> Result<(), super::create::Error> {
    diesel::update(people::table)
        .set(update)
        .execute(db_conn)
        .await?;

    Ok(())
}
