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
    api::{
        auth::{AuthUser, RemoveUnauthorizedProjects},
        extract::AuthJsonQuery,
    },
    db::{self, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub async fn index_pooled_suspensions(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
    AuthJsonQuery { mut q }: AuthJsonQuery<SuspensionQuery>,
) -> Result<Json<Vec<SuspensionSummary>>, db::Error> {
    q.filter.project_ids.remove_unauthorized_projects(&user);

    select_pooled_suspensions(id, q, &db_conn).await.map(Json)
}

pub async fn select_pooled_suspensions(
    suspension_pool_id: Uuid,
    SuspensionQuery {
        filter,
        limit,
        offset,
        order_by,
    }: SuspensionQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
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

    Ok(stmt.load(&mut db_conn).await?)
}

#[cfg(test)]
mod tests {
    use cellnoor_models::suspension::SuspensionQuery;
    use rstest::rstest;

    use super::select_pooled_suspensions;
    use crate::{
        db::DbConnection,
        test_state::{Database, N_SUSPENSIONS_PER_POOL, database, root_db_conn},
    };

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn suspension_pool_has_suspensions(
        #[future] root_db_conn: DbConnection,
        #[future] database: &'static Database,
    ) {
        let suspension_pool = &database.suspension_pools[0];

        let suspensions = select_pooled_suspensions(
            suspension_pool.id(),
            SuspensionQuery::default_with_no_limit(),
            &root_db_conn,
        )
        .await
        .unwrap();

        assert_eq!(
            suspensions.len(),
            N_SUSPENSIONS_PER_POOL,
            "found different number of suspensions in suspension pool than expected"
        );
    }
}
