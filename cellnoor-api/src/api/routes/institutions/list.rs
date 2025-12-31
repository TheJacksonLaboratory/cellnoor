use axum::{extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, InstitutionFilter, InstitutionQuery};
use cellnoor_schema::institutions::dsl::{id, name};
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

pub(super) async fn list_institutions(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<InstitutionQuery>,
) -> ApiResponse<Vec<Institution>> {
    let items = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<Institution>> for InstitutionQuery {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Vec<Institution>, db::Error> {
        let Self {
            filter,
            limit,
            offset,
            order_by,
        } = self;

        let mut stmt = Institution::query()
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

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for InstitutionFilter
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

    use cellnoor_models::institution::*;
    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;

    use crate::{
        test_state::{Database, database, root_db_conn},
        test_util::test_query,
    };

    fn sort_by_id(i1: &&Institution, i2: &&Institution) -> Ordering {
        i1.id().cmp(&i2.id())
    }

    fn sort_by_name(i1: &&Institution, i2: &&Institution) -> Ordering {
        i1.name().to_lowercase().cmp(&i2.name().to_lowercase())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_institution_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        test_query::<InstitutionQuery, _>()
            .all_records(&database.institutions)
            .sort_by(sort_by_name)
            .run(root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_institution_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        let query = InstitutionQuery::builder()
            .filter(
                InstitutionFilter::builder()
                    .names(["%a%", "%b%"].map(str::to_owned))
                    .build(),
            )
            .order_by(InstitutionOrderBy::id {
                descending: Some(false),
            })
            .order_by(InstitutionOrderBy::name {
                descending: Some(true),
            })
            .build();

        test_query()
            .all_records(&database.institutions)
            .filter(|i| {
                let s = i.name().to_lowercase();
                s.contains("a") | s.contains("b")
            })
            .sort_by(|i1, i2| sort_by_id(i1, i2).then(sort_by_name(i1, i2).reverse()))
            .db_query(query)
            .run(root_db_conn)
            .await;
    }
}
