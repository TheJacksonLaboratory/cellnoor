use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use scamplers_models::chromium_run::{GemPoolFilter, GemPoolSummary, GemsQuery};
use scamplers_schema::gem_pools;
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter},
    state::AppState,
};

pub(super) async fn list_gems(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<GemsQuery>,
) -> ApiResponse<Vec<GemPoolSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<GemPoolSummary>> for GemsQuery {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<GemPoolSummary>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = GemPoolSummary::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for GemPoolFilter
where
    gem_pools::id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self { ids } = self;
        let mut filter = BoxedFilter::new();

        if let Some(ids) = ids {
            filter = filter.and_condition(gem_pools::id.eq_any(ids));
        }

        filter
    }
}
