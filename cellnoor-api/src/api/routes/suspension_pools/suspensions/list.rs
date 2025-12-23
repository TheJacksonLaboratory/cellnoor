use axum::{extract::State, http::StatusCode};
use cellnoor_models::{
    suspension::{SuspensionQuery, SuspensionSummary},
    suspension_pool::SuspensionPoolIdSuspensions,
};
use cellnoor_schema::{suspension_tagging, suspensions};
use diesel::prelude::*;
use serde_qs::axum::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db::{self, ToBoxedFilter},
    state::AppState,
};

pub async fn list_suspensions(
    pool_id: SuspensionPoolIdSuspensions,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<SuspensionQuery>,
) -> ApiResponse<Vec<SuspensionSummary>> {
    let items = inner_handler(state, user, (pool_id, request)).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<SuspensionSummary>> for (SuspensionPoolIdSuspensions, SuspensionQuery) {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<SuspensionSummary>, db::Error> {
        let (
            pool_id,
            SuspensionQuery {
                limit,
                offset,
                filter,
                order_by,
            },
        ) = self;

        let mut stmt = suspension_tagging::table
            .filter(suspension_tagging::pool_id.eq(pool_id))
            .inner_join(suspensions::table)
            .limit(limit)
            .offset(offset)
            .select(SuspensionSummary::as_select())
            .into_boxed();

        stmt = stmt.filter(filter.to_boxed_filter());

        for ordering in order_by.as_ref() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(db_conn)?)
    }
}

#[cfg(test)]
mod tests {
    use cellnoor_models::{suspension::SuspensionQuery, suspension_pool::*};
    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;

    use crate::{
        db::Operation,
        test_state::{Database, N_SUSPENSIONS_PER_POOL, database, root_db_conn},
    };

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn suspension_pool_has_suspensions(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        let suspension_pool = &database.suspension_pools[0];

        let query = (
            SuspensionPoolIdSuspensions(suspension_pool.id()),
            SuspensionQuery::default_with_no_limit(),
        );

        let suspensions = root_db_conn
            .interact(|db_conn| query.execute(db_conn).unwrap())
            .await
            .unwrap();

        assert_eq!(
            suspensions.len(),
            N_SUSPENSIONS_PER_POOL,
            "found different number of suspensions in suspension pool than expected"
        );
    }
}
