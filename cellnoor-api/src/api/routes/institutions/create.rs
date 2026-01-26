use axum::{Json, extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, NewInstitution};
use cellnoor_schema::institutions::dsl::institutions;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{
    api::{self, auth, routes::ApiResponse},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_institution(
    _: State<AppState>,
    db_conn: DbConnection,
    Json(institution): Json<NewInstitution>,
) -> ApiResponse<Institution> {
    insert_institution(institution, db_conn)
        .await
        .map(|i| (StatusCode::CREATED, Json(i)))
}

pub async fn insert_institution(
    institution: NewInstitution,
    mut db_conn: DbConnection,
) -> Result<Institution, api::Error> {
    Ok(diesel::insert_into(institutions)
        .values(institution)
        .returning(Institution::as_returning())
        .get_result(&mut db_conn)
        .await?)
}
