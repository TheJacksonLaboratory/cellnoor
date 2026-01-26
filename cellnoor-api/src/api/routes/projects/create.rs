use cellnoor_models::project::{Project, ProjectCreation};
use cellnoor_schema::projects;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;

use crate::api::{self, AuthenticatedUser};

impl api::AuthorizedRequest<Project> for ProjectCreation {
    type ValidationData = ();

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), api::DataError> {
        // We don't need to validate that `started_at` < `ended_at` because the database validates that already
        Ok(())
    }

    async fn handle(
        self,
        mut db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Project, api::Error> {
        Ok(diesel::insert_into(projects::table)
            .values(self)
            .returning(Project::as_returning())
            .get_result(&mut db_conn)
            .await?)
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

    fn authorize(self, user: AuthenticatedUser) -> Result<Self::Authorized, api::auth::Error> {
        user.authorize_admin_only()?;

        Ok(self)
    }
}
