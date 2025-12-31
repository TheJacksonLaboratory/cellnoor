use axum::{extract::State, http::StatusCode};
use cellnoor_models::lab::{Lab, LabCreation};
use cellnoor_schema::labs::dsl::{id, labs};
use diesel::{RunQueryDsl, prelude::*};
use uuid::Uuid;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_lab(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<LabCreation>,
) -> ApiResponse<Lab> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Lab> for LabCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Lab, db::Error> {
        let created_id: Uuid = diesel::insert_into(labs)
            .values(self)
            .returning(id)
            .get_result(db_conn)?;

        Ok(Lab::query().filter(id.eq(created_id)).first(db_conn)?)
    }
}
