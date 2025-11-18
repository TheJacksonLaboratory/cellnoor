use axum::{extract::State, http::StatusCode};
use diesel::{RunQueryDsl, prelude::*};
use scamplers_models::lab::{self, Lab};
use scamplers_schema::labs;
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
    ValidJson(request): ValidJson<lab::Creation>,
) -> ApiResponse<Lab> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Lab> for lab::Creation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Lab, db::Error> {
        let id: Uuid = diesel::insert_into(labs::table)
            .values(self)
            .returning(labs::id)
            .get_result(db_conn)?;

        Ok(Lab::query().filter(labs::id.eq(id)).first(db_conn)?)
    }
}
