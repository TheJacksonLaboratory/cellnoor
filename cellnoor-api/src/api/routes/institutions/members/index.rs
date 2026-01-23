use axum::{extract::State, http::StatusCode};
use cellnoor_models::{
    institution::{self, InstitutionId},
    person::{self, PersonFilter, PersonQuery, PersonSummary},
};
use diesel_async::AsyncPgConnection;

use crate::{
    api::{
        self,
        auth::{self, Authorization},
        extract::{QsQuery, auth::AuthenticatedUser},
        request::{AuthorizedRequest, Request},
    },
    db::{self},
    state::AppState,
};

impl AuthorizedRequest<Vec<PersonSummary>> for (InstitutionId, PersonQuery) {
    type ValidationData = ();

    fn validate(&self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(
        self,
        _db_conn: &AsyncPgConnection,
    ) -> Result<Vec<person::PersonSummary>, api::Error> {
        let (institution_id, mut person_query) = self;
        person_query.filter.institution_ids = institution_id.into();

        todo!()
    }
}

impl Request<Vec<PersonSummary>> for (InstitutionId, PersonQuery) {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(&self, _db_conn: &AsyncPgConnection) -> Result<(), db::Error> {
        Ok(())
    }

    fn authorize(self, _authorization: Authorization) -> Result<Self, auth::Error> {
        Ok(self)
    }
}
