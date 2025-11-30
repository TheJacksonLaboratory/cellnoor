use axum::extract::State;
use diesel::prelude::*;
use reqwest::StatusCode;
use scamplers_models::suspension::{NucleusSuspensionCreation, Suspension, SuspensionContent};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{
            ApiResponse, Root, inner_handler,
            suspensions::create::{insert_suspension, insert_suspension_preparers},
        },
    },
    db,
    state::AppState,
};

pub(super) async fn create_nucleus_suspension(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<NucleusSuspensionCreation>,
) -> ApiResponse<Suspension> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Suspension> for NucleusSuspensionCreation {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Suspension, db::Error> {
        let Self {
            inner,
            preparer_ids,
        } = self;

        let suspension_id = insert_suspension(inner, SuspensionContent::Cells, db_conn)?;

        insert_suspension_preparers(suspension_id, &preparer_ids, db_conn)?;

        suspension_id.execute(db_conn)
    }
}
