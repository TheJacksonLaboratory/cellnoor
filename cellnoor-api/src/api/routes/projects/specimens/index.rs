use cellnoor_models::project::ProjectId;
use cellnoor_models::specimen::{SpecimenQuery, SpecimenSummary};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::api::{AuthenticatedUser, auth};
use crate::{api, db::ToBoxedFilter};

impl api::AuthorizedRequest<Vec<SpecimenSummary>> for (ProjectId, SpecimenQuery) {
    type ValidationData = ();

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(
        self,
        mut db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Vec<SpecimenSummary>, api::Error> {
        let (
            project_id,
            SpecimenQuery {
                mut filter,
                limit,
                offset,
                order_by,
            },
        ) = self;

        filter.projects = project_id.into();

        let mut stmt = SpecimenSummary::query()
            .limit(limit)
            .offset(offset)
            .filter(filter.to_boxed_filter())
            .into_boxed();

        for ordering in order_by.as_ref() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(&mut db_conn).await?)
    }
}

impl api::Request<Vec<SpecimenSummary>> for (ProjectId, SpecimenQuery) {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        Ok(())
    }

    fn authorize(self, user: AuthenticatedUser) -> Result<Self::Authorized, api::auth::Error> {
        let (project_id, specimen_query) = self;

        let authorized_projects = user
            .authorized_projects(project_id.into())
            .expect("this should be `Some` because we are passing in a project");

        if !authorized_projects.contains(project_id.as_ref()) {
            return Err(auth::Error::PermissionDenied);
        }

        Ok((project_id, specimen_query))
    }
}
