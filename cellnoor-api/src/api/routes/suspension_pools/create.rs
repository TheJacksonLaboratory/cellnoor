use axum::extract::State;
use cellnoor_models::suspension_pool::{
    SuspensionPool, SuspensionPoolCreation, SuspensionPoolId, SuspensionTagging,
};
use cellnoor_schema::{suspension_pool_preparers, suspension_pools, suspension_tagging};
use diesel::prelude::*;
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_suspension_pool(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SuspensionPoolCreation>,
) -> ApiResponse<SuspensionPool> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SuspensionPool> for SuspensionPoolCreation {
    fn execute(self, db_conn: &mut PgConnection) -> Result<SuspensionPool, db::Error> {
        let SuspensionPoolCreation {
            inner,
            preparer_ids,
            suspensions,
        } = self;

        let suspension_pool: SuspensionPool = diesel::insert_into(suspension_pools::table)
            .values(inner)
            .returning(SuspensionPool::as_returning())
            .get_result(db_conn)?;
        let suspension_pool_id = suspension_pool.id().into();

        insert_suspension_pool_preparers(suspension_pool_id, preparer_ids.as_ref(), db_conn)?;
        insert_suspension_tags(suspension_pool_id, suspensions.as_ref(), db_conn)?;

        Ok(suspension_pool)
    }
}

fn insert_suspension_pool_preparers(
    pool_id: SuspensionPoolId,
    preparer_ids: &[Uuid],
    db_conn: &mut PgConnection,
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
        .execute(db_conn)?;

    Ok(())
}

fn insert_suspension_tags(
    pool_id: SuspensionPoolId,
    taggings: &[SuspensionTagging],
    db_conn: &mut PgConnection,
) -> Result<(), db::Error> {
    let tag_mappings: Vec<_> = taggings
        .iter()
        .map(|t| (suspension_tagging::pool_id.eq(pool_id), t))
        .collect();

    diesel::insert_into(suspension_tagging::table)
        .values(tag_mappings)
        .execute(db_conn)?;

    Ok(())
}
