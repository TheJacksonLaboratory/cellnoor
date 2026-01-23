use cellnoor_schema::{people, project_people, projects};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashSet;

use cellnoor_models::{
    chromium_dataset::{ChromiumDatasetQuery, ChromiumDatasetSummary},
    person::PersonId,
};
use uuid::Uuid;

use crate::{
    api::{self, routes::chromium_datasets::chromium_datasets_to_all_specimens},
    db::{BoxedFilterExt, ToBoxedFilter},
};

impl api::AuthorizedRequest<Vec<ChromiumDatasetSummary>> for (PersonId, ChromiumDatasetQuery) {
    type ValidationData = ();

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(
        self,
        mut db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Vec<ChromiumDatasetSummary>, api::Error> {
        let (person_id, query) = self;
        let ChromiumDatasetQuery {
            filter,
            limit,
            offset,
            order_by,
        } = query;

        let mut stmt = chromium_datasets_to_all_specimens()
            .inner_join(projects::table.inner_join(project_people::table))
            .select(ChromiumDatasetSummary::as_select())
            .limit(limit)
            .offset(offset)
            .filter(filter.to_boxed_filter())
            .filter(project_people::person_id.eq(person_id))
            .into_boxed();

        for ordering in order_by.as_ref() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(&mut db_conn).await?)
    }
}

impl api::Request<Vec<ChromiumDatasetSummary>> for (PersonId, ChromiumDatasetQuery) {
    type Authorized = (PersonId, ChromiumDatasetQuery);
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        Ok(())
    }

    fn authorize(
        self,
        authorization: api::auth::Authorization,
    ) -> Result<Self::Authorized, api::auth::Error> {
        let (person_id, mut query) = self;

        query.filter.project_ids = authorization.authorized_projects(query.filter.project_ids);

        Ok((person_id, query))
    }
}
