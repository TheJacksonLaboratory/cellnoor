use axum::{extract::State, http::StatusCode};
use cellnoor_models::suspension::{
    NucleusSuspensionCreation, Suspension, SuspensionContent, SuspensionId,
};
use cellnoor_schema::suspensions;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        extract::{Json, auth::AuthenticatedUser},
        routes::{
            ApiResponse, Root, handle_api_request, suspensions::create::insert_suspension_preparers,
        },
    },
    db,
    state::AppState,
};

pub(super) async fn create_nucleus_suspension(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<NucleusSuspensionCreation>,
) -> ApiResponse<Suspension> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Suspension> for NucleusSuspensionCreation {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Suspension, db::Error> {
        let preparer_ids = self.common().preparer_ids().to_vec();

        let suspension_id: SuspensionId = diesel::insert_into(suspensions::table)
            .values((self, suspensions::content.eq(SuspensionContent::Nuclei)))
            .returning(suspensions::id)
            .get_result(db_conn)?;

        insert_suspension_preparers(suspension_id, &preparer_ids, db_conn)?;

        suspension_id.execute(db_conn)
    }
}
