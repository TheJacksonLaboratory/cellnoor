use axum::{extract::State, http::StatusCode};
use cellnoor_models::lab::{LabFilter, LabQuery, LabSummary};
use cellnoor_schema::labs::dsl::{id, name};
use diesel::{SelectableExpression, prelude::*};
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter, utils::like_any},
    state::AppState,
};

pub(super) async fn list_labs(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<LabQuery>,
) -> ApiResponse<Vec<LabSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<LabSummary>> for LabQuery {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<LabSummary>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = LabSummary::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for LabFilter
where
    id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self { ids, names } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(names) = names {
            filter = filter.and_condition(like_any(name, names));
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use cellnoor_models::lab::*;
    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;

    use crate::{
        test_state::{Database, database, root_db_conn},
        test_util::test_query,
    };

    fn sort_by_name(i1: &&LabSummary, i2: &&LabSummary) -> Ordering {
        i1.name().to_lowercase().cmp(&i2.name().to_lowercase())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_lab_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        test_query::<LabQuery, _>()
            .all_records(&database.labs)
            .sort_by(sort_by_name)
            .run(root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_lab_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        let query = LabQuery::builder()
            .filter(
                LabFilter::builder()
                    .names(["%l%", "%a%", "%b%"].map(str::to_owned))
                    .build(),
            )
            .limit(i64::MAX)
            .order_by(LabOrderBy::name {
                descending: Some(true),
            })
            .build();

        test_query()
            .all_records(&database.labs)
            .filter(|i| {
                let s = i.name().to_lowercase();
                s.contains("l") | s.contains("a") | s.contains("b")
            })
            .sort_by(|i1, i2| sort_by_name(i1, i2).reverse())
            .db_query(query)
            .run(root_db_conn)
            .await;
    }
}
