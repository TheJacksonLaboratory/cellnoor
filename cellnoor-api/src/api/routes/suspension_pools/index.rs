use axum::{Json, extract::State};
use cellnoor_models::suspension_pool::{SuspensionPool, SuspensionPoolFilter, SuspensionPoolQuery};
use cellnoor_schema::suspension_pools::{id, name, pooled_at, project_id, readable_id};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff_diesel::ToDiesel;

use crate::{
    api::{
        auth::RemoveUnauthorizedProjects,
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter, like_any},
    state::AppState,
};

pub async fn index_suspension_pools(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<SuspensionPoolQuery>,
) -> Result<Json<Vec<SuspensionPool>>, db::Error> {
    select_suspension_pools(q, &db_conn).await.map(Json)
}

pub async fn select_suspension_pools(
    SuspensionPoolQuery {
        filter,
        limit,
        offset,
        order_by,
    }: SuspensionPoolQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SuspensionPool>, db::Error> {
    let mut stmt = SuspensionPool::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for SuspensionPoolFilter
where
    id: SelectableExpression<QS>,
    readable_id: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
    pooled_at: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            ids,
            readable_ids,
            project_ids,
            names,
            pooled_before,
            pooled_after,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(readable_ids) = readable_ids {
            filter = filter.and_condition(readable_id.eq_any(readable_ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        if let Some(names) = names {
            filter = filter.and_condition(like_any(name, names));
        }

        if let Some(pooled_before) = pooled_before.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(pooled_at.lt(pooled_before));
        }

        if let Some(pooled_after) = pooled_after.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(pooled_at.gt(pooled_after));
        }

        filter
    }
}

impl Authorize for SuspensionPoolQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
