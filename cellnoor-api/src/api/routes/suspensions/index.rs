use axum::{Json, extract::State};
use cellnoor_models::suspension::{SuspensionFilter, SuspensionQuery, SuspensionSummary};
use cellnoor_schema::suspensions::{id, project_id};
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
    project_id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
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

impl Authorize for SuspensionQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
