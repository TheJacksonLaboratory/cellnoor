use cellnoor_schema::{people, project_people, projects};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashSet;

use cellnoor_models::{
    chromium_dataset::{ChromiumDatasetQuery, ChromiumDatasetSummary},
    person::PersonId,
    project::{Project, ProjectQuery},
    specimen::{SpecimenQuery, SpecimenSummary},
};
use uuid::Uuid;

use crate::{
    api::{self, AuthenticatedUser, routes::chromium_datasets::chromium_datasets_to_all_specimens},
    db::{BoxedFilterExt, ToBoxedFilter},
};

impl api::AuthorizedRequest<Vec<SpecimenSummary>> for (PersonId, SpecimenQuery) {
    type ValidationData = ();

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(
        self,
        mut db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Vec<SpecimenSummary>, api::Error> {
        let (person_id, query) = self;
        let SpecimenQuery {
            filter,
            limit,
            offset,
            order_by,
        } = query;

        todo!();

        // let mut stmt = chromium_datasets_to_all_specimens()
        //     .inner_join(projects::table.inner_join(project_people::table))
        //     .select(ChromiumDatasetSummary::as_select())
        //     .limit(limit)
        //     .offset(offset)
        //     .filter(filter.to_boxed_filter())
        //     .filter(project_people::person_id.eq(person_id))
        //     .into_boxed();

        // for ordering in order_by {
        //     stmt = stmt.then_order_by(ordering);
        // }

        // Ok(stmt.load(&mut db_conn).await?)
    }
}

impl api::Request<Vec<SpecimenSummary>> for (PersonId, SpecimenQuery) {
    type Authorized = (PersonId, SpecimenQuery);
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        Ok(())
    }

    fn authorize(self, user: AuthenticatedUser) -> Result<Self::Authorized, api::auth::Error> {
        todo!()
    }
}
