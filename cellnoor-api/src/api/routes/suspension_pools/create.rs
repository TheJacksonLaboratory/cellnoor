use axum::{Json, extract::State};
use cellnoor_models::{
    suspension::SuspensionContent,
    suspension_pool::{NewSuspensionPool, SuspensionPool, SuspensionPoolFields, SuspensionTagging},
};
use cellnoor_schema::{
    specimens, suspension_pool_preparers, suspension_pools, suspension_tagging, suspensions,
};
use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use jiff::Timestamp;
use non_empty::NonEmptyVec;
use uuid::Uuid;

use crate::{
    api::{routes::suspensions::create::SpecimenInfo, util::validate_timestamps},
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn create_suspension_pool(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(NewSuspensionPool {
        inner: suspension_pool,
        preparer_ids,
        suspensions: pooled_suspensions,
    }): Json<NewSuspensionPool>,
) -> Result<Json<SuspensionPool>, db::Error> {
    let suspension_info = suspension_info(
        pooled_suspensions
            .as_ref()
            .iter()
            .map(SuspensionTagging::suspension_id),
        &mut db_conn,
    )
    .await?;

    validate_all_suspensions_have_same_contents(&suspension_info)?;
    validate_all_suspensions_have_same_project(&suspension_info)?;
    validate_suspensions_created_or_received_before_pooled(
        &suspension_info,
        suspension_pool.pooled_at(),
    )?;

    let suspension_pool = db_conn
        .transaction(move |db_conn| {
            insert_suspension_pool_and_preparers_and_tags(
                suspension_info.get(0).map(|s| s.project_id).unwrap(),
                suspension_pool,
                preparer_ids,
                pooled_suspensions,
                db_conn,
            )
            .scope_boxed()
        })
        .await?;

    Ok(Json(suspension_pool))
}

async fn insert_suspension_pool_and_preparers_and_tags(
    project_id: Uuid,
    suspension_pool: SuspensionPoolFields,
    preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
    pooled_suspensions: NonEmptyVec<SuspensionTagging, { usize::MAX }>,
    db_conn: &mut DbConnection,
) -> Result<SuspensionPool, db::Error> {
    let suspension_pool = insert_suspension_pool(project_id, suspension_pool, db_conn).await?;

    insert_suspension_pool_preparers(suspension_pool.id(), preparer_ids.as_ref(), db_conn).await?;
    insert_suspension_tags(suspension_pool.id(), pooled_suspensions.as_ref(), db_conn).await?;

    Ok(suspension_pool)
}

pub(super) async fn insert_suspension_pool(
    project_id: Uuid,
    suspension_pool: SuspensionPoolFields,
    db_conn: &mut DbConnection,
) -> Result<SuspensionPool, db::Error> {
    Ok(diesel::insert_into(suspension_pools::table)
        .values((suspension_pool, suspension_pools::project_id.eq(project_id)))
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

#[derive(HasQuery)]
#[diesel(check_for_backend(Pg), table_name=suspensions, base_query=suspensions::table.inner_join(specimens::table))]
struct SuspensionInfo {
    content: SuspensionContent,
    #[diesel(deserialize_as=jiff_diesel::NullableTimestamp)]
    created_at: Option<Timestamp>,
    project_id: Uuid,
    #[diesel(embed)]
    specimen_info: SpecimenInfo,
}

async fn suspension_info(
    suspension_ids: impl Iterator<Item = Uuid>,
    db_conn: &mut DbConnection,
) -> Result<Vec<SuspensionInfo>, db::Error> {
    Ok(SuspensionInfo::query()
        .filter(suspensions::id.eq_any(suspension_ids))
        .load(db_conn)
        .await?)
}

fn validate_suspensions_created_or_received_before_pooled(
    suspension_info: &[SuspensionInfo],
    pooled_at: Timestamp,
) -> Result<(), db::DataError> {
    for (timestamp, field_name) in suspension_info.iter().map(|s| {
        s.created_at
            .map(|t| (t, "suspension_created_at"))
            .unwrap_or((s.specimen_info.received_at, "specimen_received_at"))
    }) {
        validate_timestamps(
            (timestamp, field_name),
            (pooled_at, "suspensions_pooled_at"),
        )?;
    }

    Ok(())
}

fn validate_all_suspensions_have_same_project(
    suspension_info: &[SuspensionInfo],
) -> Result<(), db::DataError> {
    let Some(first) = suspension_info.first() else {
        return Ok(());
    };

    if !suspension_info
        .iter()
        .all(|suspension| suspension.project_id == first.project_id)
    {
        Err(db::DataError::new_other(
            "suspensions pooled together must be part of the same project",
        ))?;
    }

    Ok(())
}

fn validate_all_suspensions_have_same_contents(
    suspension_info: &[SuspensionInfo],
) -> Result<(), db::DataError> {
    let Some(first) = suspension_info.first() else {
        return Ok(());
    };

    if !suspension_info
        .iter()
        .all(|suspension| suspension.content == first.content)
    {
        Err(db::DataError::new_other(
            "suspensions pooled together must be all either cells or nuclei",
        ))?;
    }

    Ok(())
}
