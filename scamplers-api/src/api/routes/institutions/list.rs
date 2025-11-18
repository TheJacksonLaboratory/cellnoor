use axum::{extract::State, http::StatusCode};
use diesel::{SelectableExpression, prelude::*};
use scamplers_models::institution::{self, Institution, OrdinalColumns};
use scamplers_schema::institutions;
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter},
    query,
    state::AppState,
};

pub(super) async fn list_institutions(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<institution::Query>,
) -> ApiResponse<Vec<Institution>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for institution::Filter
where
    institutions::id: SelectableExpression<QS>,
    institutions::name: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let mut filter = BoxedFilter::new();

        if let Some(ids) = self.ids() {
            filter = filter.and_condition(institutions::id.eq_any(ids));
        }

        if let Some(name) = self.name() {
            filter = filter.and_condition(institutions::name.like(name));
        }

        filter
    }
}

impl db::Operation<Vec<Institution>> for institution::Query {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Vec<Institution>, db::Error> {
        let stmt = query!(Institution::query(self).order_by(
            OrdinalColumns::Id = institutions::id,
            OrdinalColumns::Name = institutions::name
        ));

        Ok(stmt.load(db_conn)?)
    }
}
