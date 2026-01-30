use axum::{Json, extract::State};
use cellnoor_models::institution::{Institution, NewInstitution};
use cellnoor_schema::institutions::dsl::institutions;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_institution(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, db::Error> {
    insert_institution(institution, &mut db_conn)
        .await
        .map(Json)
}

pub async fn insert_institution(
    institution: NewInstitution,
    db_conn: &mut DbConnection,
) -> Result<Institution, db::Error> {
    Ok(diesel::insert_into(institutions)
        .values(institution)
        .returning(Institution::as_returning())
        .get_result(db_conn)
        .await?)
}
