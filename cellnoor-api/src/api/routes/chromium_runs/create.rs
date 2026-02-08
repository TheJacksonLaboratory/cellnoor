use axum::{Extension, Json, extract::State};
use cellnoor_models::chromium_run::{
    ChromiumRun, ChromiumRunFields, GemPoolFields, NewChromiumRun, OcmChipLoading, OcmGemPool,
    PoolMultiplexGemPool, SingleplexGemPool,
};
use cellnoor_schema::{chip_loadings, chromium_runs, specimens, suspension_pools, suspensions};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::{
        auth::AuthUser, routes::chromium_runs::show::select_chromium_run_by_id,
        util::validate_timestamps,
    },
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn create_chromium_run(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Json(chromium_run): Json<NewChromiumRun>,
) -> Result<Json<ChromiumRun>, db::Error> {
    let project_id = validate_chromium_run(&chromium_run, &mut db_conn).await?;

    let run_id = db_conn
        .transaction(move |db_conn| {
            insert_chromium_run_and_associated_data(project_id, chromium_run, db_conn).scope_boxed()
        })
        .await?;

    select_chromium_run_by_id(user.projects(), run_id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn insert_chromium_run_and_associated_data(
    project_id: Uuid,
    chromium_run: NewChromiumRun,
    db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let run_id = match chromium_run {
        NewChromiumRun::OnChipMultiplexing { inner, gem_pools } => {
            let run_id = insert_chromium_run(project_id, inner, db_conn).await?;

            let gem_pool_ids =
                insert_gem_pools(run_id, gem_pools.as_ref().iter().map(|p| &p.inner), db_conn)
                    .await?;

            insert_ocm_chip_loadings(&gem_pool_ids, gem_pools.as_ref(), db_conn).await?;

            run_id
        }
        NewChromiumRun::PoolMultiplex { inner, gem_pools } => {
            let run_id = insert_chromium_run(project_id, inner, db_conn).await?;

            let gem_pool_ids =
                insert_gem_pools(run_id, gem_pools.as_ref().iter().map(|p| &p.inner), db_conn)
                    .await?;

            insert_pool_multiplex_chip_loadings(&gem_pool_ids, gem_pools.as_ref(), db_conn).await?;

            run_id
        }
        NewChromiumRun::Singleplex { inner, gem_pools } => {
            let run_id = insert_chromium_run(project_id, inner, db_conn).await?;

            let gem_pool_ids =
                insert_gem_pools(run_id, gem_pools.as_ref().iter().map(|p| &p.inner), db_conn)
                    .await?;

            insert_singleplex_chip_loadings(&gem_pool_ids, gem_pools.as_ref(), db_conn).await?;

            run_id
        }
    };

    Ok(run_id)
}

async fn insert_chromium_run(
    project_id: Uuid,
    chromium_run: ChromiumRunFields,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    Ok(diesel::insert_into(chromium_runs::table)
        .values((chromium_runs::project_id.eq(project_id), chromium_run))
        .returning(chromium_runs::id)
        .get_result(&mut db_conn)
        .await?)
}

async fn insert_gem_pools<'a, I>(
    chromium_run_id: Uuid,
    gem_pool_data: I,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<Uuid>, db::Error>
where
    I: Iterator<Item = &'a GemPoolFields>,
{
    use cellnoor_schema::gem_pools;

    let insertions: Vec<_> = gem_pool_data
        .map(|g| (gem_pools::chromium_run_id.eq(chromium_run_id), g))
        .collect();

    Ok(diesel::insert_into(gem_pools::table)
        .values(insertions)
        .returning(gem_pools::id)
        .get_results(&mut db_conn)
        .await?)
}

async fn insert_ocm_chip_loadings(
    gem_pool_ids: &[Uuid],
    gem_pools: &[OcmGemPool],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let mut chip_loading_insertions = Vec::with_capacity(gem_pool_ids.len() * 4);

    for (gem_pool_id, gem_pool) in gem_pool_ids.iter().zip(gem_pools.as_ref()) {
        for loading in gem_pool.loading.as_ref() {
            chip_loading_insertions.push((chip_loadings::gem_pool_id.eq(gem_pool_id), loading));
        }
    }

    diesel::insert_into(chip_loadings::table)
        .values(chip_loading_insertions)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn insert_pool_multiplex_chip_loadings(
    gem_pool_ids: &[Uuid],
    gem_pools: &[PoolMultiplexGemPool],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let chip_loading_insertions: Vec<_> = gem_pool_ids
        .iter()
        .zip(gem_pools.as_ref())
        .map(|(gem_pool_id, gem_pool)| {
            (
                chip_loadings::gem_pool_id.eq(gem_pool_id),
                &gem_pool.loading,
            )
        })
        .collect();

    diesel::insert_into(chip_loadings::table)
        .values(chip_loading_insertions)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn insert_singleplex_chip_loadings(
    gem_pool_ids: &[Uuid],
    gem_pools: &[SingleplexGemPool],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
    let chip_loading_insertions: Vec<_> = gem_pool_ids
        .iter()
        .zip(gem_pools.as_ref())
        .map(|(gem_pool_id, gem_pool)| {
            (
                chip_loadings::gem_pool_id.eq(gem_pool_id),
                &gem_pool.loading,
            )
        })
        .collect();

    diesel::insert_into(chip_loadings::table)
        .values(chip_loading_insertions)
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

pub(super) async fn validate_chromium_run(
    chromium_run: &NewChromiumRun,
    db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let run_at = chromium_run.run_at();

    let suspension_ids: Vec<Uuid> = match chromium_run {
        NewChromiumRun::OnChipMultiplexing {
            inner: _,
            gem_pools,
        } => gem_pools
            .as_ref()
            .iter()
            .flat_map(|p| p.loading.as_ref().iter().map(OcmChipLoading::suspension_id))
            .collect(),
        NewChromiumRun::PoolMultiplex {
            inner: _,
            gem_pools,
        } => {
            let suspension_pool_ids = gem_pools
                .as_ref()
                .iter()
                .map(|p| p.loading.suspension_pool_id())
                .collect();

            return validate_loaded_suspension_pools(suspension_pool_ids, run_at, db_conn).await;
        }
        NewChromiumRun::Singleplex {
            inner: _,
            gem_pools,
        } => gem_pools
            .as_ref()
            .iter()
            .map(|p| p.loading.suspension_id())
            .collect(),
    };

    validate_loaded_suspensions(&suspension_ids, run_at, db_conn).await
}

async fn validate_loaded_suspension_pools(
    pool_ids: Vec<Uuid>,
    chromium_run_at: Timestamp,
    db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let project_ids_and_pooling_times = suspension_pools_info(&pool_ids, db_conn).await?;

    for (_, pooled_at) in &project_ids_and_pooling_times {
        validate_timestamps(
            (*pooled_at, "suspensions_pooled_at"),
            (chromium_run_at, "chromium_run_at"),
        )?;
    }

    Ok(validate_all_loaded_suspensions_have_same_project(
        &project_ids_and_pooling_times,
    )?)
}

async fn validate_loaded_suspensions(
    suspension_ids: &[Uuid],
    chromium_run_at: Timestamp,
    db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let project_ids_and_suspension_creation_times =
        suspensions_info(suspension_ids, db_conn).await?;

    for (_, suspension_created_at) in &project_ids_and_suspension_creation_times {
        validate_timestamps(
            (*suspension_created_at, "suspension_created_at"),
            (chromium_run_at, "chromium_run_at"),
        )?;
    }

    Ok(validate_all_loaded_suspensions_have_same_project(
        &project_ids_and_suspension_creation_times,
    )?)
}

async fn suspensions_info(
    suspension_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<(Uuid, Timestamp)>, db::Error> {
    let timestamps = join_suspensions_to_specimens(suspension_ids)
        .select((
            suspensions::project_id,
            specimens::received_at,
            suspensions::created_at,
        ))
        .load(&mut db_conn)
        .await?;

    let timestamps = timestamps
        .into_iter()
        .map(
            |(project_id, t1, t2): (_, jiff_diesel::Timestamp, jiff_diesel::NullableTimestamp)| {
                (project_id, t2.to_jiff().unwrap_or(t1.to_jiff()))
            },
        )
        .collect();

    Ok(timestamps)
}

async fn suspension_pools_info(
    pool_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<(Uuid, Timestamp)>, db::Error> {
    let timestamps = suspension_pools::table
        .select((suspension_pools::project_id, suspension_pools::pooled_at))
        .filter(suspension_pools::id.eq_any(pool_ids))
        .load(&mut db_conn)
        .await?;

    Ok(timestamps
        .into_iter()
        .map(|(project_id, ts): (_, jiff_diesel::Timestamp)| (project_id, ts.to_jiff()))
        .collect())
}

fn validate_all_loaded_suspensions_have_same_project(
    project_ids_and_timestamps: &[(Uuid, Timestamp)],
) -> Result<Uuid, db::DataError> {
    let mut project_ids = project_ids_and_timestamps
        .iter()
        .map(|(project_id, _)| project_id);

    let Some(first_project_id) = project_ids.next() else {
        return Err(db::DataError::new_other("invalid suspension IDs"));
    };

    if !project_ids.all(|p| first_project_id == p) {
        return Err(db::DataError::new_other(
            "cannot load suspensions from different project in same Chromium run",
        ));
    }

    Ok(*first_project_id)
}

#[allow(clippy::elidable_lifetime_names)]
#[diesel::dsl::auto_type]
pub(super) fn join_suspensions_to_specimens<'a>(suspension_ids: &'a [Uuid]) -> _ {
    suspensions::table
        .inner_join(specimens::table)
        .filter(suspensions::id.eq_any(suspension_ids))
}
