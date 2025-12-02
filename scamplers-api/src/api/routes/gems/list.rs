use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use scamplers_models::chromium_run::{GemsFilter, GemsQuery, GemsSummary};
use scamplers_schema::gems;
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
) -> ApiResponse<Vec<GemsSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<GemsSummary>> for GemsQuery {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<GemsSummary>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = GemsSummary::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for GemsFilter
where
    gems::id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self { ids } = self;
        let mut filter = BoxedFilter::new();

        if let Some(ids) = ids {
            filter = filter.and_condition(gems::id.eq_any(ids));
        }

        filter
    }
}
