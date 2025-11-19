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

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;
    use scamplers_models::{OrderBy, institution::*};

    use crate::{
        test_state::{db_conn, institutions},
        test_util::test_query,
    };

    fn sort_by_name(i1: &Institution, i2: &Institution) -> Ordering {
        i1.name().cmp(i2.name())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_institution_query(
        #[future] db_conn: Connection,
        #[future] institutions: Vec<Institution>,
    ) {
        test_query::<Query, _>()
            .all_data(institutions)
            .sort_by(sort_by_name)
            .run(db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_institution_query(
        #[future] db_conn: Connection,
        #[future] institutions: Vec<Institution>,
    ) {
        let query = Query::builder()
            .filter(Filter::builder().name("institution1").build())
            .order_by(OrderBy::builder().field(OrdinalColumns::Name).build())
            .build();

        test_query()
            .all_data(institutions)
            .filter(|i| i.name().starts_with("institution1"))
            .sort_by(|i1, i2| sort_by_name(i1, i2).reverse())
            .db_query(query)
            .run(db_conn)
            .await;
    }
}
