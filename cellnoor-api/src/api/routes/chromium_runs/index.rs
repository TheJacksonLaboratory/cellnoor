use axum::{Json, extract::State};
use cellnoor_models::chromium_run::{ChromiumRunFilter, ChromiumRunQuery, ChromiumRunSummary};
use cellnoor_schema::chromium_runs::{
    additional_data, assay_id, id, project_id, readable_id, run_at, run_by, succeeded,
};
use diesel::{dsl::AssumeNotNull, prelude::*};
use diesel_async::RunQueryDsl;
use jiff_diesel::ToDiesel;

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
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<ChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRunSummary>>, db::Error> {
    select_chromium_runs(q, &db_conn).await.map(Json)
}

pub async fn select_chromium_runs(
    ChromiumRunQuery {
        filter,
        limit,
        offset,
        order_by,
    }: ChromiumRunQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<ChromiumRunSummary>, db::Error> {
    let mut stmt = ChromiumRunSummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for ChromiumRunFilter
where
    id: SelectableExpression<QS>,
    readable_id: SelectableExpression<QS>,
    assay_id: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
    run_by: SelectableExpression<QS>,
    run_at: SelectableExpression<QS>,
    succeeded: SelectableExpression<QS>,
    AssumeNotNull<additional_data>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> crate::db::BoxedFilter<'a, QS> {
        let Self {
            ids,
            readable_ids,
            assay_ids,
            project_ids,
            run_by: run_by_ids,
            run_before,
            run_after,
            succeeded: succeeded_filter,
            additional_data: additional_data_filter,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(readable_ids) = readable_ids {
            filter = filter.and_condition(readable_id.eq_any(readable_ids));
        }

        if let Some(assay_ids) = assay_ids {
            filter = filter.and_condition(assay_id.eq_any(assay_ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        if let Some(run_by_ids) = run_by_ids {
            filter = filter.and_condition(run_by.eq_any(run_by_ids));
        }

        if let Some(run_before) = run_before.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(run_at.lt(run_before));
        }

        if let Some(run_after) = run_after.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(run_at.gt(run_after));
        }

        if let Some(succeeded_filter) = succeeded_filter {
            filter = filter.and_condition(succeeded.eq(*succeeded_filter));
        }

        if let Some(additional_data_filter) = additional_data_filter {
            filter = filter.and_condition(
                additional_data
                    .assume_not_null()
                    .contains(additional_data_filter),
            );
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
