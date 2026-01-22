use axum::{extract::State, http::StatusCode};
use cellnoor_models::{
    institution::{self, InstitutionIdMembers},
    person::{self, PersonFilter, PersonQuery},
};

use crate::{
    api::{
        self,
        auth::{self, AuthorizationData},
        extract::{QsQuery, auth::AuthenticatedUser},
        routes::{ApiResponse, handle_api_request},
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
    let items = handle_api_request(state, user, (institution_id, request)).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<person::PersonSummary>> for (InstitutionIdMembers, PersonQuery) {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(&self, _db_conn: db::DbConnection) -> Result<(), db::Error> {
        Ok(())
    }

    fn authorize(self, _authorization_data: AuthorizationData) -> Result<Self, auth::Error> {
        Ok(self)
    }

    fn validate(_authorized_request: &Self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    fn execute(
        (InstitutionIdMembers(institution_id), mut person_query): Self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<person::PersonSummary>, api::Error> {
        let institution_ids = Some(vec![institution_id]);
        person_query.filter.institution_ids = institution_ids;

        PersonQuery::execute(person_query, db_conn)
    }
}
