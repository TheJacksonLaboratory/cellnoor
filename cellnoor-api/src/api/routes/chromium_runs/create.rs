use axum::{extract::State, http::StatusCode};
use cellnoor_models::chromium_run::{
    ChromiumRun, ChromiumRunCreation, ChromiumRunFields, ChromiumRunId, GemPoolFields, OcmGemPool,
    PoolMultiplexGemPool, SingleplexGemPool,
};
use cellnoor_schema::chip_loadings;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, Operation},
    state::AppState,
};

pub(super) async fn create_chromium_run(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<ChromiumRunCreation>,
) -> ApiResponse<ChromiumRun> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl Operation<ChromiumRun> for ChromiumRunCreation {
    fn execute(self, db_conn: &mut PgConnection) -> Result<ChromiumRun, db::Error> {
        let run_id = match self {
            Self::OnChipMultiplexing { inner, gem_pools } => {
                let run_id = inner.execute(db_conn)?;

                let gem_pool_ids =
                    insert_gem_pools(run_id, gem_pools.as_ref().iter().map(|p| &p.inner), db_conn)?;

                insert_ocm_chip_loadings(&gem_pool_ids, gem_pools.as_ref(), db_conn)?;

                run_id
            }
            Self::PoolMultiplex { inner, gem_pools } => {
                let run_id = inner.execute(db_conn)?;

                let gem_pool_ids =
                    insert_gem_pools(run_id, gem_pools.as_ref().iter().map(|p| &p.inner), db_conn)?;

                insert_pool_multiplex_chip_loadings(&gem_pool_ids, gem_pools.as_ref(), db_conn)?;

                run_id
            }
            Self::Singleplex { inner, gem_pools } => {
                let run_id = inner.execute(db_conn)?;

                let gem_pool_ids =
                    insert_gem_pools(run_id, gem_pools.as_ref().iter().map(|p| &p.inner), db_conn)?;

                insert_singleplex_chip_loadings(&gem_pool_ids, gem_pools.as_ref(), db_conn)?;

                run_id
            }
        };

        run_id.execute(db_conn)
    }
}

impl Operation<ChromiumRunId> for ChromiumRunFields {
    fn execute(self, db_conn: &mut PgConnection) -> Result<ChromiumRunId, db::Error> {
        use cellnoor_schema::chromium_runs::dsl::*;

        Ok(diesel::insert_into(chromium_runs)
            .values(self)
            .returning(id)
            .get_result(db_conn)?)
    }
}

fn insert_gem_pools<'a, I>(
    chromium_run_id: ChromiumRunId,
    gem_pool_data: I,
    db_conn: &mut PgConnection,
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
        .get_results(db_conn)?)
}

fn insert_ocm_chip_loadings(
    gem_pool_ids: &[Uuid],
    gem_pools: &[OcmGemPool],
    db_conn: &mut PgConnection,
) -> Result<(), db::Error> {
    let mut chip_loading_insertions = Vec::with_capacity(gem_pool_ids.len() * 4);

    for (gem_pool_id, gem_pool) in gem_pool_ids.iter().zip(gem_pools.as_ref()) {
        for loading in gem_pool.loading.as_ref() {
            chip_loading_insertions.push((chip_loadings::gem_pool_id.eq(gem_pool_id), loading));
        }
    }

    diesel::insert_into(chip_loadings::table)
        .values(chip_loading_insertions)
        .execute(db_conn)?;

    Ok(())
}

fn insert_pool_multiplex_chip_loadings(
    gem_pool_ids: &[Uuid],
    gem_pools: &[PoolMultiplexGemPool],
    db_conn: &mut PgConnection,
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
        .execute(db_conn)?;

    Ok(())
}

fn insert_singleplex_chip_loadings(
    gem_pool_ids: &[Uuid],
    gem_pools: &[SingleplexGemPool],
    db_conn: &mut PgConnection,
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
        .execute(db_conn)?;

    Ok(())
}
