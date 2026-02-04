use axum::{Json, extract::State};
use cellnoor_models::institution::{Institution, NewInstitution};
use cellnoor_schema::institutions::dsl::institutions;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_institution(
    _: State<AppState>,
    db_conn: DbConnection,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, db::Error> {
    insert_institution(institution, &db_conn).await.map(Json)
}

pub async fn insert_institution(
    institution: NewInstitution,
    mut db_conn: &AsyncPgConnection,
) -> Result<Institution, db::Error> {
    Ok(diesel::insert_into(institutions)
        .values(institution)
        .returning(Institution::as_returning())
        .get_result(&mut db_conn)
        .await?)
}
