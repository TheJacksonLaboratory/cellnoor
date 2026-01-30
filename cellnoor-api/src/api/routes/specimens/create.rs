use axum::{Json, extract::State};
use cellnoor_models::specimen::{NewSpecimen, Specimen};
use cellnoor_schema::specimens;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_specimen(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(specimen): Json<NewSpecimen>,
) -> Result<Json<Specimen>, db::Error> {
    let id = insert_specimen(specimen, &mut db_conn).await?;

    super::show::select_specimen_by_id(None, id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn insert_specimen(
    specimen: NewSpecimen,
    db_conn: &mut DbConnection,
) -> Result<Uuid, db::Error> {
    let split = match specimen {
        NewSpecimen::Block(s) => s.split_for_insertion(),
        NewSpecimen::Suspension(s) => s.split_for_insertion(),
        NewSpecimen::Tissue(s) => s.split_for_insertion(),
    };

    Ok(diesel::insert_into(specimens::table)
        .values(split)
        .returning(specimens::id)
        .get_result(db_conn)
        .await?)
}
