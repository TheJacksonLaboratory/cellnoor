use axum::{Json, extract::State};
use cellnoor_models::chromium_run::{GemPoolFilter, GemPoolQuery, GemPoolSummary};
use cellnoor_schema::gem_pools;
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

pub async fn index_gem_pools(
    _: State<AppState>,
    mut db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<GemPoolQuery>,
) -> Result<Json<Vec<GemPoolSummary>>, db::Error> {
    select_gems(q, &mut db_conn).await.map(Json)
}

pub async fn select_gems(
    GemPoolQuery {
        filter,
        limit,
        offset,
        order_by,
    }: GemPoolQuery,
    db_conn: &mut DbConnection,
) -> Result<Vec<GemPoolSummary>, db::Error> {
    let mut stmt = GemPoolSummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for GemPoolFilter
where
    gem_pools::id: SelectableExpression<QS>,
    gem_pools::project_id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self { ids, project_ids } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(gem_pools::id.eq_any(ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(gem_pools::project_id.eq_any(project_ids.iter()));
        }

        filter
    }
}

impl Authorize for GemPoolQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
