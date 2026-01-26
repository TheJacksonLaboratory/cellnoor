use cellnoor_models::institution::{Institution, InstitutionCreation};
use cellnoor_schema::institutions::dsl::institutions;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{
    api::{
        self, AuthenticatedUser,
        auth::{self},
        request::{AuthorizedRequest, Request},
    },
    db,
};

impl Request<Institution> for InstitutionCreation {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(&self, _db_conn: &AsyncPgConnection) -> Result<(), db::Error> {
        Ok(())
    }

    fn authorize(self, user: AuthenticatedUser) -> Result<InstitutionCreation, auth::Error> {
        user.authorize_admin_only()?;

        Ok(self)
    }
}

impl AuthorizedRequest<Institution> for InstitutionCreation {
    type ValidationData = ();

    fn validate(&self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(self, mut db_conn: &AsyncPgConnection) -> Result<Institution, api::Error> {
        Ok(diesel::insert_into(institutions)
            .values(self)
            .returning(Institution::as_returning())
            .get_result(&mut db_conn)
            .await?)
    }
}
