use axum::extract::State;
use cellnoor_models::suspension::{SuspensionFilter, SuspensionQuery, SuspensionSummary};
use cellnoor_schema::suspensions::id;
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

pub(super) async fn list_suspensions(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<SuspensionQuery>,
) -> ApiResponse<Vec<SuspensionSummary>> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Vec<SuspensionSummary>> for SuspensionQuery {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<SuspensionSummary>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = SuspensionSummary::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for SuspensionFilter
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
