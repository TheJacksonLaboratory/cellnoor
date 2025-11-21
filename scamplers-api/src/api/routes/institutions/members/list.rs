use diesel::prelude::*;
use scamplers_models::{
    institution::{self, members},
    person,
};
use scamplers_schema::{institutions, people};
use serde_qs::axum::QsQuery;

use crate::{api::extract::auth::AuthenticatedUser, db};

pub(super) async fn list_institutions(
    institution_id: institution::members::Id,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<person::Query>,
) -> ApiResponse<Vec<person::Summary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<person::Summary> for (institution::IdMembers, person::Query) {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<person::Summary, db::Error> {
        todo!()
    }
}
