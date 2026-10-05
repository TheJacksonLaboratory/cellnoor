use axum::{Json, extract::State};
use cellnoor_types::chromium_run::{
    ChromiumRunDetailed, ChromiumRunField,
    creation::{ChromiumRunGemWells, NewChromiumRun, NewChromiumRunRecord},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
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
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<NewChromiumRun>,
) -> Result<Json<ChromiumRunDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| insert_chromium_run(tx, record).await)
        .await
}

async fn insert_chromium_run(
    tx: &db::Transaction<'_>,
    NewChromiumRun { record, gem_wells }: NewChromiumRun,
) -> Result<ChromiumRunDetailed, DbError> {
    let run_id = insert_chromium_run_record(tx, &record).await?;

    match gem_wells {
        ChromiumRunGemWells::Standard { gem_wells } => {
            let gem_well_insertions = gem_wells
                .iter()
                .map(|g| insert_standard_gem_well(tx, g, run_id));

            futures::future::try_join_all(gem_well_insertions).await?;
        }
        ChromiumRunGemWells::OnChipMultiplexing { gem_wells } => {
            let gem_well_insertions = gem_wells.iter().map(|g| insert_ocm_gem_well(tx, g, run_id));

            futures::future::try_join_all(gem_well_insertions).await?;
        }
        ChromiumRunGemWells::Mixed { gem_wells } => {
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
) -> Result<Uuid, DbError> {
    tx.insert_returning_id(record).await
}

impl Insert for NewChromiumRunRecord {
    type Field = ChromiumRunField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
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

        vec![
            (ReadableId, readable_id),
            (AssayId, assay_id),
            (RunAt, run_at),
            (RunBy, run_by),
            (Succeeded, succeeded),
            (AdditionalData, additional_data),
        ]
    }
}

#[cfg(any(test, feature = "dev"))]
pub async fn new_record(assay_id: Uuid, run_by: Uuid) -> NewChromiumRunRecord {
    use cellnoor_types::id::NoId;
    use jiff::Timestamp;

    NewChromiumRunRecord {
        id: NoId,
        readable_id: crate::db::dummy_data::random_name_for("chromium_run"),
        assay_id,
        run_at: Timestamp::now(),
        run_by,
        succeeded: true,
        additional_data: None,
    }
}

#[cfg(any(test, feature = "dev"))]
async fn insert_test_standard_chromium_run<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewChromiumRun, ChromiumRunDetailed), DbError>
where
    F: FnMut(&mut NewChromiumRun),
{
    use cellnoor_types::{
        chromium_run::creation::{LoadedEntity, standard::NewStandardGemWell},
        nonempty::NonemptyBoundedVec,
    };

    use crate::handlers::{
        suspension_pools::create::insert_test_suspension_pool_and_suspensions,
        suspensions::create::insert_test_suspension_and_specimen,
        tenx_assays::create::insert_chromium_assays,
    };

    let (_, suspension) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
    let (_, pool) = insert_test_suspension_pool_and_suspensions(tx, |_| ()).await?;

    let person_id = suspension.preparers[0];

    let assay_id = insert_chromium_assays(tx).await?.singleplex;

    // To exercise the ability of a mulitply loaded chip, the chromium run
    // has two GEM wells
    let gem_well1 = NewStandardGemWell {
        readable_id: crate::db::dummy_data::random_name_for("gem_well"),
        loaded_entity: LoadedEntity::Suspension {
            suspension_id: *suspension.record.id,
        },
    };

    let gem_well2 = NewStandardGemWell {
        readable_id: crate::db::dummy_data::random_name_for("gem_well"),
        loaded_entity: LoadedEntity::SuspensionPool {
            suspension_pool_id: *pool.record.id,
        },
    };

    let mut new = NewChromiumRun {
        record: new_record(assay_id, person_id).await,
        gem_wells: ChromiumRunGemWells::Standard {
            gem_wells: NonemptyBoundedVec::new(vec![gem_well1, gem_well2]).unwrap(),
        },
    };

    modify(&mut new);

    let inserted = insert_chromium_run(tx, new.clone()).await?;
    Ok((new, inserted))
}

#[cfg(any(test, feature = "dev"))]
async fn insert_test_ocm_chromium_run<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewChromiumRun, ChromiumRunDetailed), DbError>
where
    F: FnMut(&mut NewChromiumRun),
{
    use cellnoor_types::{
        chromium_run::creation::{
            LoadedEntity,
            ocm::{NewOcmGemWell, OcmBarcodeId, OcmLoadedEntity},
        },
        nonempty::NonemptyBoundedVec,
    };

    use crate::handlers::{
        suspensions::create::insert_test_suspension_and_specimen,
        tenx_assays::create::insert_chromium_assays,
    };

    let (_, s1) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
    let (_, s2) = insert_test_suspension_and_specimen(tx, |_| ()).await?;

    let person_id = s1.preparers[0];

    let assay_id = insert_chromium_assays(tx).await?.ocm;

    // To exercise the ability of a mulitply loaded chip, each GEM well has
    // two suspensions, and the chromium run has two GEM wells.
    // However, this time, the two GEM wells are basically
    // equivalent so we can see if we get duplicate specimens
    let loadings = vec![
        OcmLoadedEntity {
            loaded_entity: LoadedEntity::Suspension {
                suspension_id: *s1.record.id,
            },
            ocm_barcode_id: OcmBarcodeId::Ob1,
        },
        OcmLoadedEntity {
            loaded_entity: LoadedEntity::Suspension {
                suspension_id: *s2.record.id,
            },
            ocm_barcode_id: OcmBarcodeId::Ob2,
        },
    ];
    let gem_wells = vec![
        NewOcmGemWell {
            readable_id: crate::db::dummy_data::random_name_for("gem_well"),
            loading: NonemptyBoundedVec::new(loadings.clone()).unwrap(),
        },
        NewOcmGemWell {
            readable_id: crate::db::dummy_data::random_name_for("gem_well"),
            loading: NonemptyBoundedVec::new(loadings).unwrap(),
        },
    ];
    let mut new = NewChromiumRun {
        record: new_record(assay_id, person_id).await,
        gem_wells: ChromiumRunGemWells::OnChipMultiplexing {
            gem_wells: NonemptyBoundedVec::new(gem_wells).unwrap(),
        },
    };

    modify(&mut new);

    let inserted = insert_chromium_run(tx, new.clone()).await?;
    Ok((new, inserted))
}

#[cfg(any(test, feature = "dev"))]
async fn insert_test_mixed_chromium_run<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewChromiumRun, ChromiumRunDetailed), DbError>
where
    F: FnMut(&mut NewChromiumRun),
{
    use cellnoor_types::{
        chromium_run::creation::{
            LoadedEntity,
            mixed::NewStandardOrOcmGemWell,
            ocm::{NewOcmGemWell, OcmBarcodeId, OcmLoadedEntity},
            standard::NewStandardGemWell,
        },
        nonempty::NonemptyBoundedVec,
    };

    use crate::{
        db::dummy_data::random_name_for,
        handlers::{
            suspensions::create::insert_test_suspension_and_specimen,
            tenx_assays::create::insert_chromium_assays,
        },
    };

    let (_, s1) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
    let (_, s2) = insert_test_suspension_and_specimen(tx, |_| ()).await?;

    let person_id = s1.preparers[0];

    let assay_id = insert_chromium_assays(tx).await?.ocm;

    let mut new = NewChromiumRun {
        record: new_record(assay_id, person_id).await,
        gem_wells: ChromiumRunGemWells::Mixed {
            gem_wells: NonemptyBoundedVec::new(vec![
                NewStandardOrOcmGemWell::Standard(NewStandardGemWell {
                    readable_id: random_name_for("gem_well"),
                    loaded_entity: LoadedEntity::Suspension {
                        suspension_id: *s1.record.id,
                    },
                }),
                NewStandardOrOcmGemWell::OnChipMultiplexing(NewOcmGemWell {
                    readable_id: random_name_for("gem_well"),
                    loading: NonemptyBoundedVec::new(vec![OcmLoadedEntity {
                        loaded_entity: LoadedEntity::Suspension {
                            suspension_id: *s2.record.id,
                        },
                        ocm_barcode_id: OcmBarcodeId::Ob1,
                    }])
                    .unwrap(),
                }),
            ])
            .unwrap(),
        },
    };

    modify(&mut new);

    let inserted = insert_chromium_run(tx, new.clone()).await?;
    Ok((new, inserted))
}

#[cfg(any(test, feature = "dev"))]
async fn insert_test_flex_chromium_run<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewChromiumRun, ChromiumRunDetailed), DbError>
where
    F: FnMut(&mut NewChromiumRun),
{
    use cellnoor_types::{
        chromium_run::creation::{LoadedEntity, standard::NewStandardGemWell},
        nonempty::{NonemptyBoundedVec, NonemptyVec},
        suspension_pool::{PooledSuspensions, TaggedSuspension},
    };

    use crate::handlers::{
        multiplexing_tags::create::insert_test_multiplexing_tag,
        suspension_pools::create::insert_test_suspension_pool_and_suspensions,
        suspensions::create::insert_test_suspension_and_specimen,
        tenx_assays::create::insert_chromium_assays,
    };

    let mut suspensions = Vec::with_capacity(32);
    for _ in 0..32 {
        let (_, suspension) = insert_test_suspension_and_specimen(tx, |_| ()).await?;
        let tag = insert_test_multiplexing_tag(tx).await?;

        suspensions.push(TaggedSuspension {
            suspension_id: *suspension.record.id,
            tag_id: tag.tag_id,
        });
    }

    let (_, pool) = insert_test_suspension_pool_and_suspensions(tx, |pool| {
        pool.suspensions = PooledSuspensions::FlexOligonucleotideBarcode {
            suspensions: NonemptyVec::new(suspensions.clone()).unwrap(),
        };
    })
    .await?;

    let person_id = pool.preparers[0];

    let assay_id = insert_chromium_assays(tx).await?.flex;

    let mut new = NewChromiumRun {
        record: new_record(assay_id, person_id).await,
        gem_wells: ChromiumRunGemWells::Standard {
            gem_wells: NonemptyBoundedVec::new(vec![NewStandardGemWell {
                readable_id: crate::db::dummy_data::random_name_for("gem_well"),
                loaded_entity: LoadedEntity::SuspensionPool {
                    suspension_pool_id: *pool.record.id,
                },
            }])
            .unwrap(),
        },
    };

    modify(&mut new);

    let inserted = insert_chromium_run(tx, new.clone()).await?;
    Ok((new, inserted))
}

#[cfg(any(test, feature = "dev"))]
#[derive(Clone, Copy, strum::VariantArray)]
pub enum TestChromiumRunKind {
    Standard,
    Mixed,
    OnChipMultiplexing,
    Flex,
}

#[cfg(any(test, feature = "dev"))]
pub async fn insert_test_chromium_run(
    tx: &db::Transaction<'_>,
    kind: TestChromiumRunKind,
    modify: impl FnMut(&mut NewChromiumRun),
) -> Result<ChromiumRunDetailed, DbError> {
    use TestChromiumRunKind::*;

    let (_, run) = match kind {
        Standard => insert_test_standard_chromium_run(tx, modify).await?,
        Mixed => insert_test_mixed_chromium_run(tx, modify).await?,
        OnChipMultiplexing => insert_test_ocm_chromium_run(tx, modify).await?,
        Flex => insert_test_flex_chromium_run(tx, modify).await?,
    };

    Ok(run)
}

#[cfg(test)]
pub mod test {
    use super::{
        insert_test_flex_chromium_run, insert_test_mixed_chromium_run,
        insert_test_ocm_chromium_run, insert_test_standard_chromium_run,
    };
    use crate::state::dev_util::db_client_as_admin;

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

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_flex() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_flex_chromium_run(&tx, |_| ()).await.unwrap();
    }
}
