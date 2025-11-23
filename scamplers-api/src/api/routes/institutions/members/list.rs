use axum::{extract::State, http::StatusCode};
use scamplers_models::{institution, person};
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db::{self},
    state::AppState,
};

pub async fn list_members(
    institution_id: institution::IdMembers,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<person::Query>,
) -> ApiResponse<Vec<person::Summary>> {
    let items = inner_handler(state, user, (institution_id, request)).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<person::Summary>> for (institution::IdMembers, person::Query) {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<person::Summary>, db::Error> {
        let (institution::IdMembers(institution_id), mut person_query) = self;
        person_query.set_institution_id(institution_id);

        person_query.execute(db_conn)
    }
}
