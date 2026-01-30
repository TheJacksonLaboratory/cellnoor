use aide::OperationIo;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use cellnoor_models::specimen::{NewSpecimen, Specimen};
use cellnoor_schema::specimens;
use diesel_async::RunQueryDsl;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    api::auth::AuthUser,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_specimen(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Json(specimen): Json<NewSpecimen>,
) -> Result<Json<Specimen>, db::Error> {
    validate_specimen_received_after_project_started()?;
    let id = insert_specimen(specimen, &mut db_conn).await?;

    super::show::select_specimen_by_id(user.projects(), id, &mut db_conn)
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
