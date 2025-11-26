use axum::extract::State;
use diesel::prelude::*;
use reqwest::StatusCode;
use scamplers_models::suspension::{Suspension, SuspensionCreation, SuspensionId};
use scamplers_schema::{suspension_preparers, suspensions::dsl::*};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_suspension(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SuspensionCreation>,
) -> ApiResponse<Suspension> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Suspension> for SuspensionCreation {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Suspension, db::Error> {
        let (values, preparer_ids) = self.split_for_insertion();

        let suspension_id: SuspensionId = diesel::insert_into(suspensions)
            .values(values)
            .returning(id)
            .get_result(db_conn)?;

        let preparer_mappings: Vec<_> = preparer_ids
            .into_iter()
            .map(|p| {
                (
                    suspension_preparers::suspension_id.eq(suspension_id),
                    suspension_preparers::prepared_by.eq(p),
                )
            })
            .collect();

        diesel::insert_into(suspension_preparers::table)
            .values(preparer_mappings)
            .execute(db_conn)?;

        suspension_id.execute(db_conn)
    }
}
