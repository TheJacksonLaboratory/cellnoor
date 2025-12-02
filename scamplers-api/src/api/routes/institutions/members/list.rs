use axum::{extract::State, http::StatusCode};
use scamplers_models::{
    institution::{self, InstitutionIdMembers},
    person::{self, PersonQuery},
};
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
    institution_id: institution::InstitutionIdMembers,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<person::PersonQuery>,
) -> ApiResponse<Vec<person::PersonSummary>> {
    let items = inner_handler(state, user, (institution_id, request)).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<person::PersonSummary>> for (InstitutionIdMembers, PersonQuery) {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<person::PersonSummary>, db::Error> {
        let (institution_id, mut person_query) = self;

        person_query.set_parent_id(institution_id);

        person_query.execute(db_conn)
    }
}
