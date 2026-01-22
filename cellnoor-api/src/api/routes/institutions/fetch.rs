use axum::{extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, InstitutionId};
use cellnoor_schema::institutions::dsl::id;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        self,
        auth::{self, AuthorizationData},
        extract::auth::AuthenticatedUser,
        request::{AuthorizedRequest, Request},
    },
    db::{self},
    state::AppState,
};

impl AuthorizedRequest<Institution> for InstitutionId {
    type ValidationData = ();

    fn validate(&self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    fn execute(self, db_conn: &mut PgConnection) -> Result<Institution, api::Error> {
        Ok(Institution::query().filter(id.eq(self)).first(db_conn)?)
    }
}

impl Request<Institution> for InstitutionId {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: db::DbConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn authorize(self, _authorization_data: AuthorizationData) -> Result<Self, auth::Error> {
        Ok(self)
    }
}
