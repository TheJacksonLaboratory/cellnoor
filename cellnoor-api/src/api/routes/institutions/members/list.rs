use axum::{extract::State, http::StatusCode};
use cellnoor_models::{
    institution::{self, InstitutionIdMembers},
    person::{self, PersonFilter, PersonQuery},
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
        let (InstitutionIdMembers(institution_id), mut person_query) = self;

        let institution_ids = Some(vec![institution_id]);
        if let Some(q) = &mut person_query.filter {
            q.institution_ids = institution_ids;
        } else {
            person_query.filter = Some(PersonFilter {
                institution_ids,
                ..Default::default()
            });
        }

        person_query.execute(db_conn)
    }
}
