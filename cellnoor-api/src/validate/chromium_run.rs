use cellnoor_models::{
    chromium_run::{ChromiumRunCreation, OcmChipLoading},
    suspension_pool::{SuspensionPool, SuspensionPoolFilter, SuspensionPoolQuery},
};
use cellnoor_schema::{specimens, suspensions};
use diesel::{PgConnection, prelude::*};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl Validate for ChromiumRunCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        let run_at = self.run_at();
        let suspension_ids: Vec<Uuid> = match self {
            Self::OnChipMultiplexing {
                inner: _,
                gem_pools,
            } => gem_pools
                .as_ref()
                .iter()
                .flat_map(|p| p.loading.as_ref().iter().map(OcmChipLoading::suspension_id))
                .collect(),
            Self::PoolMultiplex {
                inner: _,
                gem_pools,
            } => {
                let suspension_pool_ids = gem_pools
                    .as_ref()
                    .iter()
                    .map(|p| p.loading.suspension_pool_id())
                    .collect();

                validate_suspension_pools_created_before_chromium_run(
                    suspension_pool_ids,
                    run_at,
                    db_conn,
                )?;

                return Ok(());
            }
            Self::Singleplex {
                inner: _,
                gem_pools,
            } => gem_pools
                .as_ref()
                .iter()
                .map(|p| p.loading.suspension_id())
                .collect(),
        };

        validate_suspensions_created_before_chromium_run(&suspension_ids, run_at, db_conn)
    }
}

fn validate_suspension_pools_created_before_chromium_run(
    suspension_pool_ids: Vec<Uuid>,
    chromium_run_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let mut q = SuspensionPoolQuery::default_with_no_limit();
    q.filter = Some(SuspensionPoolFilter {
        ids: Some(suspension_pool_ids),
    });

    let suspension_pools = q.execute(db_conn)?;
    for pooled_at in suspension_pools.iter().map(SuspensionPool::pooled_at) {
        validate_timestamps(pooled_at, chromium_run_at, "run_at")?;
    }

    Ok(())
}

fn validate_suspensions_created_before_chromium_run(
    suspension_ids: &[Uuid],
    chromium_run_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let timestamps = fetch_suspension_timestamps(suspension_ids, db_conn)?;

    for ts in timestamps {
        validate_timestamps(ts, chromium_run_at, "run_at")?;
    }

    Ok(())
}

fn fetch_suspension_timestamps(
    suspension_ids: &[Uuid],
    db_conn: &mut PgConnection,
) -> Result<Vec<Timestamp>, super::Error> {
    let timestamps: Vec<(jiff_diesel::NullableTimestamp, jiff_diesel::Timestamp)> =
        join_suspensions_to_specimens(suspension_ids)
            .select((suspensions::created_at, specimens::received_at))
            .load(db_conn)?;

    Ok(timestamps
        .into_iter()
        .map(|(t1, t2)| t1.to_jiff().unwrap_or_else(|| t2.to_jiff()))
        .collect())
}

#[allow(clippy::elidable_lifetime_names)]
#[diesel::dsl::auto_type]
pub(super) fn join_suspensions_to_specimens<'a>(suspension_ids: &'a [Uuid]) -> _ {
    suspensions::table
        .inner_join(specimens::table)
        .filter(suspensions::id.eq_any(suspension_ids))
}
