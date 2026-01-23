use cellnoor_models::{
    person::PersonId,
    project::{Project, ProjectCreation, ProjectId},
};
use cellnoor_schema::{project_people, projects};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::api;

impl api::AuthorizedRequest<()> for (ProjectId, PersonId) {
    type ValidationData = ();

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), api::DataError> {
        // We don't need to validate that `started_at` < `ended_at` because the database validates that already
        Ok(())
    }

    async fn handle(self, mut db_conn: &diesel_async::AsyncPgConnection) -> Result<(), api::Error> {
        let (project_id, person_id) = self;

        diesel::insert_into(project_people::table)
            .values((
                project_people::project_id.eq(project_id),
                project_people::person_id.eq(person_id),
            ))
            .execute(&mut db_conn)
            .await?;

        Ok(())
    }
}

impl api::Request<Project> for ProjectCreation {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        Ok(())
    }

    fn authorize(
        self,
        authorization: api::auth::Authorization,
    ) -> Result<Self::Authorized, api::auth::Error> {
        authorization.authorize_admin()?;

        Ok(self)
    }
}
