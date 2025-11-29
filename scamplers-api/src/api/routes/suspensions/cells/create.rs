use axum::extract::State;
use axum_extra::routing::TypedPath;
use diesel::prelude::*;
use reqwest::StatusCode;
use scamplers_models::suspension::{CellSuspensionCreation, Suspension, SuspensionId};
use scamplers_schema::{suspension_preparers, suspensions::dsl::*};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{
            ApiResponse, Root, inner_handler,
            suspensions::create::common::insert_suspension_preparers,
        },
    },
    db,
    state::AppState,
};

pub(super) async fn create_cell_suspension(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<CellSuspensionCreation>,
) -> ApiResponse<Suspension> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Suspension> for CellSuspensionCreation {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Suspension, db::Error> {
        let Self {
            inner,
            preparer_ids,
        } = self;

        let suspension_id: SuspensionId = diesel::insert_into(suspensions)
            .values(values)
            .returning(id)
            .get_result(db_conn)?;

        insert_suspension_preparers(suspension_id);

        suspension_id.execute(db_conn)
    }
}
