use cellnoor_models::{
    person::PersonId,
    project::{Project, ProjectCreation, ProjectId},
};
use cellnoor_schema::{project_people, projects};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::api::{self, AuthenticatedUser, auth};

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

impl api::Request<()> for (ProjectId, PersonId) {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        Ok(())
    }

    fn authorize(self, user: AuthenticatedUser) -> Result<Self::Authorized, auth::Error> {
        let (project_id, person_id) = self;

        let authorized_projects = user
            .authorized_projects(project_id.into())
            .expect("there should be a project because we passed one in");

        if !authorized_projects.contains(project_id.as_ref()) {
            return Err(auth::Error::PermissionDenied);
        }

        Ok((project_id, person_id))
    }
}
