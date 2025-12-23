use axum::{extract::State, http::StatusCode};
use cellnoor_models::chromium_run::{ChromiumRunFilter, ChromiumRunQuery, ChromiumRunSummary};
use cellnoor_schema::chromium_runs::dsl::*;
use diesel::prelude::*;
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter},
    state::AppState,
};

pub(super) async fn list_chromium_runs(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<ChromiumRunQuery>,
) -> ApiResponse<Vec<ChromiumRunSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<ChromiumRunSummary>> for ChromiumRunQuery {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<ChromiumRunSummary>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = ChromiumRunSummary::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for ChromiumRunFilter
where
    id: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> crate::db::BoxedFilter<'a, QS> {
        let Self { ids } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        filter
    }
}
