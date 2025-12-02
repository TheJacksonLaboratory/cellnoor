use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use non_empty::NonEmptyVec;
use scamplers_models::chromium_run::{
    ChromiumRun, ChromiumRunCreation, ChromiumRunFields, ChromiumRunId, GemsFields, OcmGems,
    PoolMultiplexGems, SingleplexGems,
};
use scamplers_schema::chip_loadings;
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
            Self::OnChipMultiplexing { inner, gems } => {
                let run_id = inner.execute(db_conn)?;

                let gems_id = (
                    run_id,
                    gems.as_ref()
                        .iter()
                        .map(|OcmGems { inner, loading: _ }| inner),
                )
                    .execute(db_conn)?;

                let loadings: Vec<_> = gems
                    .as_ref()
                    .iter()
                    .map(|OcmGems { inner: _, loading }| loading)
                    .flat_map(NonEmptyVec::as_ref)
                    .map(|l| (chip_loadings::gems_id.eq(gems_id), l))
                    .collect();

                diesel::insert_into(chip_loadings::table)
                    .values(loadings)
                    .execute(db_conn)?;

                run_id
            }
            Self::PoolMultiplex { inner, gems } => {
                let run_id = inner.execute(db_conn)?;

                let gems_id = (
                    run_id,
                    gems.as_ref()
                        .iter()
                        .map(|PoolMultiplexGems { inner, loading: _ }| inner),
                )
                    .execute(db_conn)?;

                let loadings: Vec<_> = gems
                    .as_ref()
                    .iter()
                    .map(|PoolMultiplexGems { inner: _, loading }| {
                        (chip_loadings::gems_id.eq(gems_id), loading)
                    })
                    .collect();

                diesel::insert_into(chip_loadings::table)
                    .values(loadings)
                    .execute(db_conn)?;

                run_id
            }
            Self::Singleplex { inner, gems } => {
                let run_id = inner.execute(db_conn)?;

                let gems_id = (
                    run_id,
                    gems.as_ref()
                        .iter()
                        .map(|SingleplexGems { inner, loading: _ }| inner),
                )
                    .execute(db_conn)?;

                let loadings: Vec<_> = gems
                    .as_ref()
                    .iter()
                    .map(|SingleplexGems { inner: _, loading }| {
                        (chip_loadings::gems_id.eq(gems_id), loading)
                    })
                    .collect();

                diesel::insert_into(chip_loadings::table)
                    .values(loadings)
                    .execute(db_conn)?;

                run_id
            }
        };

        run_id.execute(db_conn)
    }
}

impl Operation<ChromiumRunId> for ChromiumRunFields {
    fn execute(self, db_conn: &mut PgConnection) -> Result<ChromiumRunId, db::Error> {
        use scamplers_schema::chromium_runs::dsl::*;

        Ok(diesel::insert_into(chromium_runs)
            .values(self)
            .returning(id)
            .get_result(db_conn)?)
    }
}

impl<'a, I> db::Operation<Uuid> for (ChromiumRunId, I)
where
    I: Iterator<Item = &'a GemsFields>,
{
    fn execute(self, db_conn: &mut PgConnection) -> Result<Uuid, db::Error> {
        use scamplers_schema::gems::dsl::*;

        let (run_id, gems_data) = self;
        let insertions: Vec<_> = gems_data.map(|g| (chromium_run_id.eq(run_id), g)).collect();

        Ok(diesel::insert_into(gems)
            .values(insertions)
            .returning(id)
            .get_result(db_conn)?)
    }
}
