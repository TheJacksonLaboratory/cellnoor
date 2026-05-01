#![allow(clippy::get_first)]
use axum::{Json, extract::State};
use cellnoor_models::{
    suspension::SuspensionContent,
    suspension_pool::{NewSuspensionPool, SuspensionPool, SuspensionPoolFields, SuspensionTagging},
};
use cellnoor_schema::{
    specimens, suspension_pool_preparers, suspension_pooling, suspension_pools, suspensions,
};
use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::{routes::suspensions::create::SpecimenInfo, util::validate_timestamps},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_suspension_pool(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(suspension_pool): Json<NewSuspensionPool>,
) -> Result<Json<SuspensionPool>, db::Error> {
    let (inner, pooled_suspensions): (_, Vec<_>) = match &suspension_pool {
        NewSuspensionPool::ExogenousTag {
            inner, suspensions, ..
        } => (
            inner,
            suspensions
                .as_ref()
                .iter()
                .map(SuspensionTagging::suspension_id)
                .collect(),
        ),
        NewSuspensionPool::Genetic {
            inner, suspensions, ..
        } => (inner, suspensions.as_ref().to_vec()),
    };

    let suspension_info = suspension_info(&pooled_suspensions, &db_conn).await?;

    if suspension_info.is_empty() {
        return Err(db::Error::InvalidReference {
            resource: "suspension_pools".to_owned(),
            referenced_resource: "suspension".to_owned(),
            value: Some(pooled_suspensions[0].to_string()),
        });
    }
    let project_id = suspension_info[0].project_id;

    validate_all_suspensions_have_same_contents(&suspension_info)?;
    validate_all_suspensions_have_same_project(&suspension_info)?;
    validate_suspensions_created_or_received_before_pooled(&suspension_info, inner.pooled_at())?;

    let suspension_pool = db_conn
        .transaction(move |db_conn| {
            insert_suspension_pool_and_preparers_and_pool_mapping(
                project_id,
                suspension_pool,
                db_conn,
            )
            .scope_boxed()
        })
        .await?;

    Ok(Json(suspension_pool))
}

pub async fn insert_suspension_pool_and_preparers_and_pool_mapping(
    project_id: Uuid,
    suspension_pool: NewSuspensionPool,
    db_conn: &diesel_async::AsyncPgConnection,
) -> Result<SuspensionPool, db::Error> {
    let multiplexing_tag_type = suspension_pool.into_str();
    let (inner, preparers, tagged_suspensions, untagged_suspensions) =
        suspension_pool.split_for_insertion();

    let suspension_pool =
        insert_suspension_pool(project_id, inner, multiplexing_tag_type, db_conn).await?;

    insert_suspension_pool_preparers(suspension_pool.id(), preparers.as_ref(), db_conn).await?;

    if let Some(tagged_suspensions) = tagged_suspensions {
        insert_suspension_tags(suspension_pool.id(), tagged_suspensions.as_ref(), db_conn).await?;
    } else if let Some(untagged_suspensions) = untagged_suspensions {
        insert_suspension_pool_mapping(
            suspension_pool.id(),
            untagged_suspensions.as_ref(),
            db_conn,
        )
        .await?;
    }

    Ok(suspension_pool)
}

pub(super) async fn insert_suspension_pool(
    project_id: Uuid,
    suspension_pool: SuspensionPoolFields,
    multiplexing_tag_type: &str,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<SuspensionPool, db::Error> {
    Ok(diesel::insert_into(suspension_pools::table)
        .values((
            suspension_pool,
            suspension_pools::multiplexing_type.eq(multiplexing_tag_type),
            suspension_pools::project_id.eq(project_id),
        ))
        .returning(SuspensionPool::as_returning())
        .get_result(&mut db_conn)
        .await?)
}

async fn insert_suspension_pool_preparers(
    pool_id: Uuid,
    preparer_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
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
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn insert_suspension_tags(
    pool_id: Uuid,
    taggings: &[SuspensionTagging],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let tag_mappings: Vec<_> = taggings
        .iter()
        .map(|t| (suspension_pooling::pool_id.eq(pool_id), t))
        .collect();

    diesel::insert_into(suspension_pooling::table)
        .values(tag_mappings)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn insert_suspension_pool_mapping(
    pool_id: Uuid,
    suspensions: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let mappings: Vec<_> = suspensions
        .iter()
        .map(|suspension_id| {
            (
                suspension_pooling::pool_id.eq(pool_id),
                suspension_pooling::suspension_id.eq(suspension_id),
            )
        })
        .collect();

    diesel::insert_into(suspension_pooling::table)
        .values(mappings)
        .execute(&mut db_conn)
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
    suspension_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<SuspensionInfo>, db::Error> {
    Ok(SuspensionInfo::query()
        .filter(suspensions::id.eq_any(suspension_ids))
        .load(&mut db_conn)
        .await?)
}

fn validate_suspensions_created_or_received_before_pooled(
    suspension_info: &[SuspensionInfo],
    pooled_at: Timestamp,
) -> Result<(), db::DataError> {
    for (timestamp, field_name) in suspension_info.iter().map(|s| {
        s.created_at
            .map_or((s.specimen_info.received_at, "specimen_received_at"), |t| {
                (t, "suspension_created_at")
            })
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
