use axum::{extract::State, http::StatusCode};
use diesel::{SelectableExpression, prelude::*};
use scamplers_models::lab::{self, LabSummary};
use scamplers_schema::labs;
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter, utils::like_any},
    query,
    state::AppState,
};

pub(super) async fn list_labs(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<lab::Query>,
) -> ApiResponse<Vec<LabSummary>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<LabSummary>> for lab::Query {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<LabSummary>, db::Error> {
        use lab::OrdinalColumns::{Id, Name};

        let stmt = query!(LabSummary::query(self).order_by(Id = labs::id, Name = labs::name));

        Ok(stmt.load(db_conn)?)
    }
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for lab::Filter
where
    labs::id: SelectableExpression<QS>,
    labs::name: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let mut filter = BoxedFilter::new();

        if let Some(ids) = self.ids() {
            filter = filter.and_condition(labs::id.eq_any(ids));
        }

        if let Some(names) = self.names() {
            filter = filter.and_condition(like_any(labs::name, names));
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;
    use scamplers_models::lab::*;

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
        test_query::<Query, _>()
            .all_data(&database.labs)
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
        let query = Query::builder()
            .filter(
                Filter::builder()
                    .names(["%l%", "%a%", "%b%"].map(str::to_owned))
                    .build(),
            )
            .order_by_descending(OrdinalColumns::Name)
            .build();

        test_query()
            .all_data(&database.labs)
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
