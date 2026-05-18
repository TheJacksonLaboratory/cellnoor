use axum::{Json as AxumJson, extract::State};
use cellnoor_types::chromium_run::{ChromiumRun, NewChromiumRunRecord, creation::NewChromiumRun};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs, insert_into},
    error::{Error, ErrorInner},
    handlers::chromium_runs::{
        create::gem_well::{insert_mixed_gem_well, insert_ocm_gem_well, insert_standard_gem_well},
        show::select_chromium_run_by_id,
    },
    state::AppState,
};

mod gem_well;

pub async fn create_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    AxumJson(record): AxumJson<NewChromiumRun>,
) -> Result<AxumJson<ChromiumRun>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_chromium_run(&tx, record).await.map(AxumJson)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_chromium_run(
    tx: &db::Transaction<'_>,
    new: NewChromiumRun,
) -> Result<ChromiumRun, ErrorInner> {
    // We destructure twice cus it's so much less repetitive
    let run_id = match &new {
        NewChromiumRun::Standard { common, .. }
        | NewChromiumRun::OnChipMultiplexing { common, .. }
        | NewChromiumRun::Mixed { common, .. } => insert_chromium_run_record(tx, common).await?,
    };

    match new {
        NewChromiumRun::Standard { gem_wells, .. } => {
            let gem_well_insertions = gem_wells
                .iter()
                .map(|g| insert_standard_gem_well(tx, g, run_id));

            futures::future::try_join_all(gem_well_insertions).await?;
        }
        NewChromiumRun::OnChipMultiplexing { gem_wells, .. } => {
            let gem_well_insertions = gem_wells.iter().map(|g| insert_ocm_gem_well(tx, g, run_id));

            futures::future::try_join_all(gem_well_insertions).await?;
        }
        NewChromiumRun::Mixed { gem_wells, .. } => {
            let gem_well_insertions = gem_wells
                .iter()
                .map(|g| insert_mixed_gem_well(tx, g, run_id));

            futures::future::try_join_all(gem_well_insertions).await?;
        }
    }

    select_chromium_run_by_id(tx, run_id).await
}

async fn insert_chromium_run_record(
    tx: &db::Transaction<'_>,
    record: &NewChromiumRunRecord,
) -> Result<Uuid, ErrorInner> {
    Ok(insert_into(tx, "chromium_run", record).await?)
}

impl AsFieldValuePairs<&'static str, 6> for NewChromiumRunRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 6> {
        let Self {
            id: _,
            readable_id,
            assay_id,
            run_at,
            run_by,
            succeeded,
            additional_data,
        } = self;

        [
            ("readable_id", readable_id),
            ("assay_id", assay_id),
            ("run_at", run_at),
            ("run_by", run_by),
            ("succeeded", succeeded),
            ("additional_data", additional_data),
        ]
    }
}
