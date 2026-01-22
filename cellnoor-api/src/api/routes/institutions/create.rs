use cellnoor_models::institution::{Institution, InstitutionCreation};
use cellnoor_schema::institutions::dsl::institutions;
use diesel::prelude::*;

use crate::{
    api::{
        self,
        auth::{self, AuthorizationData},
        extract::Json,
        request::{AuthorizedRequest, Request},
    },
    db,
};

impl Request<Institution> for InstitutionCreation {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(&self, _db_conn: db::DbConnection) -> Result<(), db::Error> {
        Ok(())
    }

    fn authorize(
        self,
        authorization_data: AuthorizationData,
    ) -> Result<InstitutionCreation, auth::Error> {
        if !authorization_data.is_admin() {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(self)
    }
}

impl AuthorizedRequest<Institution> for InstitutionCreation {
    type ValidationData = ();

    fn validate(&self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    fn execute(self, db_conn: &mut PgConnection) -> Result<Institution, api::Error> {
        Ok(diesel::insert_into(institutions)
            .values(self)
            .returning(Institution::as_returning())
            .get_result(db_conn)?)
    }
}
