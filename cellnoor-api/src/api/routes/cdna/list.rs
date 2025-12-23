use axum::{extract::State, http::StatusCode};
use cellnoor_models::cdna::{CdnaFilter, CdnaQuery, CdnaSummary};
use cellnoor_schema::cdna::dsl::id;
use diesel::{SelectableExpression, prelude::*};

use crate::{
    api::{
        extract::{auth::AuthenticatedUser, query::QsQuery},
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter},
    state::AppState,
};

#[axum::debug_handler]
pub(super) async fn list_cdna(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<CdnaQuery>,
) -> ApiResponse<Vec<CdnaSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<CdnaSummary>> for CdnaQuery {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<CdnaSummary>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = CdnaSummary::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for CdnaFilter
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
