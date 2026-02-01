use axum::{
    Extension, Json,
    extract::{State, rejection::ExtensionRejection},
    http::StatusCode,
};
use cellnoor_models::suspension_pool::{
    NewSuspensionPool, SuspensionPool, SuspensionPoolFields, SuspensionTagging,
};
use cellnoor_schema::{suspension_pool_preparers, suspension_pools, suspension_tagging};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::AuthUser,
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn create_suspension_pool(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(NewSuspensionPool {
        inner: suspension_pool,
        preparer_ids,
        suspensions,
    }): Json<NewSuspensionPool>,
) -> Result<Json<SuspensionPool>, db::Error> {
    todo!("validate the suspension content");
    todo!("validate that the suspension was pooled after the suspensions were creatd");
    todo!("insert the preparers");
    todo!("insert the tagging");
    let suspension_pool = insert_suspension_pool(suspension_pool, &mut db_conn).await?;
    let suspension_pool_id = suspension_pool.id();

    tokio::try_join!(
        insert_suspension_pool_preparers(suspension_pool_id, preparer_ids.as_ref(), &mut db_conn),
        insert_suspension_tags(suspension_pool_id, suspensions.as_ref(), &mut db_conn)
    );

    Ok(Json(suspension_pool))
}

pub(super) async fn insert_suspension_pool(
    suspension_pool: SuspensionPoolFields,
    db_conn: &mut DbConnection,
) -> Result<SuspensionPool, db::Error> {
    Ok(diesel::insert_into(suspension_pools::table)
        .values(suspension_pool)
        .returning(SuspensionPool::as_returning())
        .get_result(db_conn)
        .await?)
}

async fn insert_suspension_pool_preparers(
    pool_id: Uuid,
    preparer_ids: &[Uuid],
    db_conn: &mut DbConnection,
) -> Result<(), db::Error> {
    let preparer_mappings: Vec<_> = preparer_ids
        .iter()
        .map(|p| {
            (
                suspension_pool_preparers::pool_id.eq(pool_id),
                suspension_pool_preparers::prepared_by.eq(p),
            )
        })
        .collect();

    diesel::insert_into(suspension_pool_preparers::table)
        .values(preparer_mappings)
        .execute(db_conn)
        .await?;

    Ok(())
}

async fn insert_suspension_tags(
    pool_id: Uuid,
    taggings: &[SuspensionTagging],
    db_conn: &mut DbConnection,
) -> Result<(), db::Error> {
    let tag_mappings: Vec<_> = taggings
        .iter()
        .map(|t| (suspension_tagging::pool_id.eq(pool_id), t))
        .collect();

    diesel::insert_into(suspension_tagging::table)
        .values(tag_mappings)
        .execute(db_conn)
        .await?;

    Ok(())
}
