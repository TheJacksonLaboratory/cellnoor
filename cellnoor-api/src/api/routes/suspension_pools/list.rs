use axum::extract::State;
use cellnoor_models::suspension_pool::{SuspensionPool, SuspensionPoolFilter, SuspensionPoolQuery};
use cellnoor_schema::suspension_pools::id;
use diesel::prelude::*;
use reqwest::StatusCode;
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter},
    state::AppState,
};

pub(super) async fn list_suspension_pools(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<SuspensionPoolQuery>,
) -> ApiResponse<Vec<SuspensionPool>> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Vec<SuspensionPool>> for SuspensionPoolQuery {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<SuspensionPool>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = SuspensionPool::query()
            .limit(limit)
            .offset(offset)
            .filter(filter.to_boxed_filter())
            .into_boxed();

        for ordering in order_by.as_ref() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(db_conn)?)
    }
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for SuspensionPoolFilter
where
    id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self { ids } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        filter
    }
}
