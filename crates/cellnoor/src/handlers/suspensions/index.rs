use axum::{Json, extract::State};
use cellnoor_models::suspension::{SuspensionFilter, SuspensionQuery, SuspensionSummary};
use cellnoor_schema::suspensions::{
    additional_data, content, created_at, id, lysis_duration_minutes, parent_specimen_id,
    project_id, readable_id, target_cell_recovery,
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

pub async fn index_suspensions(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<SuspensionQuery>,
) -> Result<Json<Vec<SuspensionSummary>>, db::Error> {
    select_suspensions(q, &db_conn).await.map(Json)
}

pub async fn select_suspensions(
    SuspensionQuery {
        filter,
        limit,
        offset,
        order_by,
    }: SuspensionQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SuspensionSummary>, db::Error> {
    let mut stmt = SuspensionSummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for SuspensionFilter
where
    id: SelectableExpression<QS>,
    readable_id: SelectableExpression<QS>,
    parent_specimen_id: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
    content: SelectableExpression<QS>,
    AssumeNotNull<created_at>: SelectableExpression<QS>,
    AssumeNotNull<lysis_duration_minutes>: SelectableExpression<QS>,
    AssumeNotNull<target_cell_recovery>: SelectableExpression<QS>,
    AssumeNotNull<additional_data>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            ids,
            readable_ids,
            parent_specimen_ids,
            project_ids,
            contents,
            created_before,
            created_after,
            lysis_duration_less_than,
            lysis_duration_more_than,
            target_cell_recovery_less_than,
            target_cell_recovery_more_than,
            additional_data: additional_data_filter,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(readable_ids) = readable_ids {
            filter = filter.and_condition(readable_id.eq_any(readable_ids));
        }

        if let Some(parent_specimen_ids) = parent_specimen_ids {
            filter = filter.and_condition(parent_specimen_id.eq_any(parent_specimen_ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        if let Some(contents) = contents {
            filter = filter.and_condition(content.eq_any(contents));
        }

        if let Some(created_before) = created_before.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(created_at.assume_not_null().lt(created_before));
        }

        if let Some(created_after) = created_after.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(created_at.assume_not_null().gt(created_after));
        }

        if let Some(lysis_duration_less_than) = lysis_duration_less_than {
            filter = filter.and_condition(
                lysis_duration_minutes
                    .assume_not_null()
                    .lt(*lysis_duration_less_than),
            );
        }

        if let Some(lysis_duration_more_than) = lysis_duration_more_than {
            filter = filter.and_condition(
                lysis_duration_minutes
                    .assume_not_null()
                    .gt(*lysis_duration_more_than),
            );
        }

        if let Some(target_cell_recovery_less_than) = target_cell_recovery_less_than {
            filter = filter.and_condition(
                target_cell_recovery
                    .assume_not_null()
                    .lt(*target_cell_recovery_less_than),
            );
        }

        if let Some(target_cell_recovery_more_than) = target_cell_recovery_more_than {
            filter = filter.and_condition(
                target_cell_recovery
                    .assume_not_null()
                    .gt(*target_cell_recovery_more_than),
            );
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

impl Authorize for SuspensionQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
