use cellnoor_models::project::{Project, ProjectFilter, ProjectQuery};
use cellnoor_schema::projects::dsl::*;
use diesel::SelectableExpression;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff_diesel::ToDiesel;

use crate::{
    api,
    db::{BoxedFilter, BoxedFilterExt, ToBoxedFilter, like_any},
};

impl api::AuthorizedRequest<Vec<Project>> for ProjectQuery {
    type ValidationData = ();

    fn validate(&self, validation_data: Self::ValidationData) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(
        self,
        mut db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Vec<Project>, api::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = Project::query()
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

impl api::Request<Vec<Project>> for ProjectQuery {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        db_conn: &diesel_async::AsyncPgConnection,
    ) -> Result<Self::ValidationData, crate::db::Error> {
        Ok(())
    }

    fn authorize(
        mut self,
        authorization: api::auth::Authorization,
    ) -> Result<Self::Authorized, api::auth::Error> {
        self.filter.ids = authorization.authorized_projects(self.filter.ids);

        Ok(self)
    }
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for ProjectFilter
where
    id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            ids,
            names,
            started_before,
            started_after,
            ended_before,
            ended_after,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(names) = names {
            filter = filter.and_condition(like_any(name, names));
        }

        for timestamp in [started_before, ended_before] {
            let Some(timestamp) = timestamp else {
                continue;
            };

            filter = filter.and_condition(started_at.lt(timestamp.to_diesel()));
        }

        for timestamp in [started_after, ended_after] {
            let Some(timestamp) = timestamp else {
                continue;
            };

            filter = filter.and_condition(ended_at.gt(timestamp.to_diesel()));
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use cellnoor_models::project::*;
    use rstest::rstest;

    use crate::{
        db::DbConnection,
        test_state::{Database, database, root_db_conn},
        test_util::test_query,
    };

    fn sort_by_name(i1: &&Project, i2: &&Project) -> Ordering {
        i1.name().to_lowercase().cmp(&i2.name().to_lowercase())
    }

    #[rstest]
    #[awt]
    #[tokio::test(flavor = "multi_thread")]
    async fn default_project_query(
        #[future] root_db_conn: DbConnection,
        #[future] database: &'static Database,
    ) {
        test_query::<ProjectQuery, _>()
            .all_records(&database.projects)
            .sort_by(sort_by_name)
            .run(root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test(flavor = "multi_thread")]
    async fn specific_project_query(
        #[future] root_db_conn: DbConnection,
        #[future] database: &'static Database,
    ) {
        let query = ProjectQuery::builder()
            .filter(
                ProjectFilter::builder()
                    .names(["%l%", "%a%", "%b%"].map(str::to_owned))
                    .build(),
            )
            .limit(i64::MAX)
            .order_by(ProjectOrderBy::name {
                descending: Some(true),
            })
            .build();

        test_query()
            .all_records(&database.projects)
            .filter(|i| {
                let s = i.name().to_lowercase();
                s.contains("l") | s.contains("a") | s.contains("b")
            })
            .sort_by(|i1, i2| sort_by_name(i1, i2).reverse())
            .db_query(query)
            .run(root_db_conn)
            .await;
    }
}
