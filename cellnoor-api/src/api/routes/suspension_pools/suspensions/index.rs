use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    suspension::{SuspensionQuery, SuspensionSummary},
};
use cellnoor_schema::{suspension_tagging, suspensions};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::{auth::AuthUser, extract::AuthJsonQuery},
    db::{self, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub async fn index_pooled_suspensions(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
    AuthJsonQuery { q }: AuthJsonQuery<SuspensionQuery>,
) -> Result<Json<Vec<SuspensionSummary>>, db::Error> {
    select_pooled_suspensions(id, q, &mut db_conn)
        .await
        .map(Json)
}

pub async fn select_pooled_suspensions(
    suspension_pool_id: Uuid,
    SuspensionQuery {
        filter,
        limit,
        offset,
        order_by,
    }: SuspensionQuery,
    db_conn: &mut DbConnection,
) -> Result<Vec<SuspensionSummary>, db::Error> {
    let mut stmt = suspension_tagging::table
        .filter(suspension_tagging::pool_id.eq(suspension_pool_id))
        .inner_join(suspensions::table)
        .limit(limit)
        .offset(offset)
        .select(SuspensionSummary::as_select())
        .into_boxed();

    stmt = stmt.filter(filter.to_boxed_filter());

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(db_conn).await?)
}

#[cfg(test)]
mod tests {
    // use cellnoor_models::{suspension::SuspensionQuery, suspension_pool::*};
    // use rstest::rstest;

    // use crate::{
    //     db::DbConnection,
    //     test_state::{Database, N_SUSPENSIONS_PER_POOL, database, root_db_conn},
    // };

    // #[rstest]
    // #[awt]
    // #[tokio::test(flavor = "multi_thread")]
    // async fn suspension_pool_has_suspensions(
    //     #[future] root_db_conn: DbConnection,
    //     #[future] database: &'static Database,
    // ) {
    //     let suspension_pool = &database.suspension_pools[0];

    //     let query = (
    //         SuspensionPoolIdSuspensions(suspension_pool.id()),
    //         SuspensionQuery::default_with_no_limit(),
    //     );

    //     let suspensions = root_db_conn
    //         .interact(|db_conn| query.execute(db_conn).unwrap())
    //         .await
    //         .unwrap();

    //     assert_eq!(
    //         suspensions.len(),
    //         N_SUSPENSIONS_PER_POOL,
    //         "found different number of suspensions in suspension pool than expected"
    //     );
    // }
}
