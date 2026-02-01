use axum::{Json, extract::State, http::StatusCode};
use cellnoor_models::chromium_run::{ChromiumRunFilter, ChromiumRunQuery, ChromiumRunSummary};
use cellnoor_schema::chromium_runs::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    api::{
        auth::RemoveUnauthorizedProjects,
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub(super) async fn index_chromium_runs(
    _: State<AppState>,
    mut db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRunSummary>>, db::Error> {
    select_chromium_runs(q, &mut db_conn).await.map(Json)
}

pub async fn select_chromium_runs(
    ChromiumRunQuery {
        filter,
        limit,
        offset,
        order_by,
    }: ChromiumRunQuery,
    db_conn: &mut DbConnection,
) -> Result<Vec<ChromiumRunSummary>, db::Error> {
    let mut stmt = ChromiumRunSummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for ChromiumRunFilter
where
    id: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> crate::db::BoxedFilter<'a, QS> {
        let Self { ids, project_ids } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        filter
    }
}

impl Authorize for ChromiumRunQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
