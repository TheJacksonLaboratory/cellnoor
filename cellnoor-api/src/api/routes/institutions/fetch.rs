use axum::{extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, InstitutionId};
use cellnoor_schema::institutions::dsl::id;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

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

    async fn handle(self, mut db_conn: &AsyncPgConnection) -> Result<Institution, api::Error> {
        Ok(Institution::query()
            .filter(id.eq(self))
            .first(&mut db_conn)
            .await?)
    }
}

impl Request<Institution> for InstitutionId {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: &AsyncPgConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn authorize(self, _authorization_data: AuthorizationData) -> Result<Self, auth::Error> {
        Ok(self)
    }
}
