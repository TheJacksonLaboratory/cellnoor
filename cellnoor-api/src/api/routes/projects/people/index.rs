use cellnoor_models::person::{PersonQuery, PersonSummary};
use cellnoor_models::project::ProjectId;
use cellnoor_models::specimen::{SpecimenQuery, SpecimenSummary};
use cellnoor_schema::project_people;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::api::{AuthenticatedUser, auth};
use crate::{api, db::ToBoxedFilter};

impl api::AuthorizedRequest<Vec<PersonSummary>> for (ProjectId, PersonQuery) {
    type ValidationData = ();

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(
        self,
        mut db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Vec<PersonSummary>, api::Error> {
        let (
            project_id,
            PersonQuery {
                filter,
                limit,
                offset,
                order_by,
            },
        ) = self;

        let mut stmt = PersonSummary::query()
            .inner_join(project_people::table)
            .limit(limit)
            .offset(offset)
            .filter(project_people::project_id.eq(project_id))
            .filter(filter.to_boxed_filter())
            .into_boxed();

        for ordering in order_by.as_ref() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(&mut db_conn).await?)
    }
}

impl api::Request<Vec<PersonSummary>> for (ProjectId, PersonQuery) {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        Ok(())
    }

    fn authorize(self, user: AuthenticatedUser) -> Result<Self::Authorized, api::auth::Error> {
        let (project_id, person_query) = self;

        let authorized_projects = user
            .authorized_projects(project_id.into())
            .expect("there should be a project because we passed one in");

        if !authorized_projects.contains(&project_id.as_ref()) {
            return Err(auth::Error::PermissionDenied);
        }

        Ok((project_id, person_query))
    }
}
