use axum::{Json as AxumJson, extract::State};
use cellnoor_types::chromium_run::{
    ChromiumRunDetailed, ChromiumRunField, NewChromiumRunRecord, creation::NewChromiumRun,
};
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
) -> Result<AxumJson<ChromiumRunDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_chromium_run(&tx, record).await.map(AxumJson)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_chromium_run(
    tx: &db::Transaction<'_>,
    new: NewChromiumRun,
) -> Result<ChromiumRunDetailed, ErrorInner> {
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

impl AsFieldValuePairs<ChromiumRunField, 6> for NewChromiumRunRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, ChromiumRunField, 6> {
        use ChromiumRunField::*;

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
            (ReadableId, readable_id),
            (AssayId, assay_id),
            (RunAt, run_at),
            (RunBy, run_by),
            (Succeeded, succeeded),
            (AdditionalData, additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        chromium_run::{
            ChromiumRunDetailed, LoadingVolume, NewChromiumRunRecord,
            creation::{
                NewChipLoadingCommonFields, NewChromiumRun,
                mixed::{NewMixedChipLoading, NewMixedGemWell},
                ocm::{NewOcmChipLoading, NewOcmGemWell, OcmBarcodeId},
                standard::{NewStandardChipLoading, NewStandardGemWell},
            },
        },
        id::NoId,
        units::Microliter,
    };
    use jiff::Timestamp;
    use nonempty::NonemptyBoundedVec;
    use positive::PositiveF32;
    use postgres_types::Json;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            chromium_runs::create::insert_chromium_run,
            suspension_pools::create::test::insert_test_suspension_pool_and_suspensions,
            suspensions::create::test::insert_test_suspension_and_specimen,
            tenx_assays::create::insert_test_chromium_assay,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub fn loading_common() -> NewChipLoadingCommonFields {
        NewChipLoadingCommonFields {
            suspension_volume_loaded: Json(LoadingVolume {
                value: PositiveF32::new(50.0).unwrap(),
                unit: Microliter::Microliter,
            }),
            buffer_volume_loaded: Json(LoadingVolume {
                value: PositiveF32::new(50.0).unwrap(),
                unit: Microliter::Microliter,
            }),
            additional_data: None,
        }
    }

    pub fn new_common(assay_id: Uuid, run_by: Uuid) -> NewChromiumRunRecord {
        NewChromiumRunRecord {
            id: NoId {},
            readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
            assay_id,
            run_at: Timestamp::now(),
            run_by,
            succeeded: true,
            additional_data: None,
        }
    }

    pub async fn insert_test_standard_chromium_run<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewChromiumRun, ChromiumRunDetailed), ErrorInner>
    where
        F: FnMut(&mut NewChromiumRun),
    {
        let (_, pool1) = insert_test_suspension_pool_and_suspensions(tx, |_| ()).await?;
        let (_, pool2) = insert_test_suspension_pool_and_suspensions(tx, |_| ()).await?;

        let person_id = pool1.preparers[0];

        let (_, assay) = insert_test_chromium_assay(tx).await?;
        let assay_id = assay.id;

        // To exercise the ability of a mulitply loaded chip, the chromium run has two
        // GEM wells
        let gem_well1 = NewStandardGemWell {
            readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
            loading: NewStandardChipLoading::SuspensionPool {
                suspension_pool_id: *pool1.record.id,
                common: loading_common(),
            },
        };

        let gem_well2 = NewStandardGemWell {
            readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
            loading: NewStandardChipLoading::SuspensionPool {
                suspension_pool_id: *pool2.record.id,
                common: loading_common(),
            },
        };

        let mut new = NewChromiumRun::Standard {
            common: new_common(assay_id, person_id),
            gem_wells: NonemptyBoundedVec::new(vec![gem_well1, gem_well2]).unwrap(),
        };

        modify(&mut new);

        let inserted = insert_chromium_run(tx, new.clone()).await?;
        Ok((new, inserted))
    }

    pub async fn insert_test_ocm_chromium_run<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewChromiumRun, ChromiumRunDetailed), ErrorInner>
    where
        F: FnMut(&mut NewChromiumRun),
    {
        let (_, s1) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
        let (_, s2) = insert_test_suspension_and_specimen(tx, |_| ()).await?;

        let person_id = s1.preparers[0];

        let (_, assay) = insert_test_chromium_assay(tx).await?;
        let assay_id = assay.id;

        // To exercise the ability of a mulitply loaded chip, each GEM well has two
        // suspensions, and the chromium run has two GEM wells. However, this time, the
        // two GEM wells are basically equivalent so we can see if we get duplicate
        // specimens
        let loadings = vec![
            NewOcmChipLoading::Suspension {
                suspension_id: *s1.record.id,
                common: loading_common(),
                ocm_barcode_id: OcmBarcodeId::Ob1,
            },
            NewOcmChipLoading::Suspension {
                suspension_id: *s2.record.id,
                common: loading_common(),
                ocm_barcode_id: OcmBarcodeId::Ob2,
            },
        ];
        let gem_wells = vec![
            NewOcmGemWell {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NonemptyBoundedVec::new(loadings.clone()).unwrap(),
            },
            NewOcmGemWell {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                loading: NonemptyBoundedVec::new(loadings).unwrap(),
            },
        ];
        let mut new = NewChromiumRun::OnChipMultiplexing {
            common: new_common(assay_id, person_id),
            gem_wells: NonemptyBoundedVec::new(gem_wells).unwrap(),
        };

        modify(&mut new);

        let inserted = insert_chromium_run(tx, new.clone()).await?;
        Ok((new, inserted))
    }

    pub async fn insert_test_mixed_chromium_run<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewChromiumRun, ChromiumRunDetailed), ErrorInner>
    where
        F: FnMut(&mut NewChromiumRun),
    {
        let (_, s1) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
        let (_, s2) = insert_test_suspension_and_specimen(tx, |_| ()).await?;

        let person_id = s1.preparers[0];

        let (_, assay) = insert_test_chromium_assay(tx).await?;
        let assay_id = assay.id;

        let mut new = NewChromiumRun::Mixed {
            common: new_common(assay_id, person_id),
            gem_wells: NonemptyBoundedVec::new(vec![
                NewMixedGemWell {
                    readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                    loading: NewMixedChipLoading::Standard(NewStandardChipLoading::Suspension {
                        suspension_id: *s1.record.id,
                        common: loading_common(),
                    }),
                },
                NewMixedGemWell {
                    readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                    loading: NewMixedChipLoading::Ocm(
                        NonemptyBoundedVec::new(vec![NewOcmChipLoading::Suspension {
                            suspension_id: *s2.record.id,
                            common: loading_common(),
                            ocm_barcode_id: OcmBarcodeId::Ob1,
                        }])
                        .unwrap(),
                    ),
                },
            ])
            .unwrap(),
        };

        modify(&mut new);

        let inserted = insert_chromium_run(tx, new.clone()).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_standard() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_standard_chromium_run(&tx, |_| ())
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_ocm() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_ocm_chromium_run(&tx, |_| ()).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_mixed() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_mixed_chromium_run(&tx, |_| ()).await.unwrap();
    }
}
