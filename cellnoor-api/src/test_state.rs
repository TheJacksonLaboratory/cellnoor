use std::ops::Range;

use cellnoor_models::{
    cdna::{CdnaCreation, CdnaFields, CdnaQuery, CdnaSummary},
    chromium_dataset::{ChromiumDatasetCreation, ChromiumDatasetQuery, ChromiumDatasetSummary},
    chromium_run::{
        ChipLoadingFields, ChromiumRunCreation, ChromiumRunFields, GemPoolFields, GemPoolQuery,
        GemPoolSummary, MAX_GEM_POOLS_PER_NON_OCM_RUN, MAX_GEM_POOLS_PER_OCM_RUN,
        MAX_SUSPENSIONS_PER_OCM_GEM_POOL, OcmBarcodeId, OcmChipLoading, OcmGemPool,
        PoolMultiplexChipLoading, PoolMultiplexGemPool, SingleplexChipLoading, SingleplexGemPool,
        Volume,
    },
    generic_query,
    institution::{Institution, InstitutionCreation, InstitutionQuery},
    lab::{LabCreation, LabFields, LabQuery, LabSummary},
    library::{LibraryCreation, LibraryFields, LibraryQuery, LibrarySummary},
    multiplexing_tag::MultiplexingTag,
    person::{PersonCreation, PersonFields, PersonQuery, PersonSummary},
    specimen::{
        BlockFixative, CryopreservedSuspensionCreation, CryopreservedTissueCreation,
        FixedBlockCreation, FixedBlockEmbeddingMatrix, FixedTissueCreation, FrozenBlockCreation,
        FrozenBlockEmbeddingMatrix, FrozenSuspensionCreation, FrozenTissueCreation, Species,
        SpecimenCommonFields, SpecimenCreation, SpecimenQuery, SpecimenSummary, TissueFixative,
    },
    suspension::{
        SuspensionContent, SuspensionCreation, SuspensionFields, SuspensionQuery, SuspensionSummary,
    },
    suspension_pool::{
        SuspensionPool, SuspensionPoolCreation, SuspensionPoolFields, SuspensionPoolQuery,
        SuspensionTagging,
    },
    tenx_assay::{LibraryType, SampleMultiplexing, TenxAssayFilter, TenxAssayQuery},
};
use deadpool_diesel::postgres::{Connection, Pool};
use diesel::prelude::*;
use jiff::Timestamp;
use non_empty::{NonEmptyString, NonEmptyVec};
use pretty_assertions::assert_eq;
use rand::{
    Rng,
    distr::Alphanumeric,
    seq::{IndexedRandom, IteratorRandom},
};
use ranged::{RangedU16, RangedU32};
use rstest::fixture;
use serde_json::json;
use strum::VariantArray;
use tokio::{sync::OnceCell, task::JoinSet};
use uuid::Uuid;

use crate::{
    config::Config,
    db,
    db::Operation,
    state::{AppState, create_test_db_pool},
};

static TEST_STATE: OnceCell<TestState> = OnceCell::const_new();
static DATABASE: OnceCell<Database> = OnceCell::const_new();

#[fixture]
pub async fn database() -> &'static Database {
    let state = TEST_STATE.get_or_init(TestState::new).await;
    DATABASE.get_or_init(|| Database::new(state)).await
}

#[fixture]
pub async fn root_db_conn() -> Connection {
    let state = TEST_STATE.get_or_init(TestState::new).await;
    state.root_db_conn().await
}

pub struct TestState {
    _inner: AppState,
    root_db_pool: Pool,
}

impl TestState {
    async fn new() -> Self {
        let config = Config::read()
            .expect("test configuration should be readable from environment variables");

        Self {
            _inner: AppState::initialize(&config)
                .await
                .expect("should be able to initialize app state"),
            root_db_pool: create_test_db_pool(&config.db_root_url()).unwrap(),
        }
    }

    async fn populate_db(&'static self) {
        // This is a safeguard so that a failure to initialize test state doesn't cause
        // endless repetition
        let institution_ids = self
            .all_extract::<InstitutionQuery, _, _, _>(Institution::id)
            .await;
        if institution_ids.len() > 1 {
            return;
        }
        self.insert_institutions().await;
        self.insert_people().await;
        self.insert_labs().await;
        self.insert_specimens().await;
        self.insert_suspensions().await;
        self.insert_suspension_pools().await;
        self.insert_singleplex_chromium_runs().await;
        self.insert_ocm_chromium_runs().await;
        self.insert_pool_multiplex_chromium_runs().await;
        self.insert_cdna().await;
        self.insert_libraries().await;
        self.insert_chromium_datasets().await;
    }

    async fn insert_institutions(&'static self) {
        let join_set: JoinSet<_> = (0..N_INSTITUTIONS)
            .map(|_| self.insert_random_institution())
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_institution(&self) {
        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(|db_conn| {
                InstitutionCreation::new(Uuid::now_v7(), random_non_empty_string())
                    .execute(db_conn)
                    .unwrap()
            })
            .await
            .unwrap();
    }

    async fn insert_people(&'static self) {
        let institution_ids = self
            .all_extract::<InstitutionQuery, _, _, _>(Institution::id)
            .await;

        let mut join_set = JoinSet::new();
        // Skip the first institution since that already has a person
        for inst in &institution_ids[1..] {
            for _ in 0..N_PEOPLE_PER_INSTITUTION {
                join_set.spawn(self.insert_random_person(*inst));
            }
        }

        join_set.join_all().await;
    }

    async fn insert_random_person(&self, institution_id: Uuid) {
        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(move |db_conn| {
                let name = random_string();
                let email = format!("{name}@example.com");

                PersonCreation::builder()
                    .inner(
                        PersonFields::builder()
                            .name(NonEmptyString::new(name).unwrap())
                            .institution_id(institution_id)
                            .build(),
                    )
                    .email(NonEmptyString::new(email).unwrap())
                    .roles([])
                    .build()
                    .execute(db_conn)
                    .unwrap();
            })
            .await
            .unwrap();
    }

    async fn insert_labs(&'static self) {
        let people_ids = self.all_people_ids().await;

        let join_set: JoinSet<_> = (0..N_LABS)
            .map(|i| self.insert_random_lab(people_ids[i]))
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_lab(&self, pi_id: Uuid) {
        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(move |db_conn| {
                LabCreation::builder()
                    .inner(
                        LabFields::builder()
                            .name(random_non_empty_string())
                            .delivery_dir(random_non_empty_string())
                            .pi_id(pi_id)
                            .build(),
                    )
                    .build()
                    .execute(db_conn)
                    .unwrap();
            })
            .await
            .unwrap();
    }

    async fn insert_specimens(&'static self) {
        let people_ids = self.all_people_ids().await;
        let lab_ids = self.all_extract::<LabQuery, _, _, _>(LabSummary::id).await;

        let mut join_set = JoinSet::new();
        let mut counter = 0;
        // Skip the first person
        for person_id in &people_ids[1..] {
            for lab_id in lab_ids.iter().copied().take(N_SPECIMENS_PER_PERSON) {
                counter += 1;
                join_set.spawn(self.insert_random_specimen(counter, *person_id, lab_id));
            }
        }

        join_set.join_all().await;
    }

    async fn insert_random_specimen(&self, i: usize, submitted_by: Uuid, lab_id: Uuid) {
        let db_conn = self.root_db_conn().await;

        let inner = SpecimenCommonFields::builder()
            .readable_id(random_non_empty_string())
            .name(random_non_empty_string())
            .submitted_by(submitted_by)
            .lab_id(lab_id)
            .received_at(random_time())
            .species(Species::VARIANTS.choose_unwrap())
            .tissue(random_non_empty_string())
            .additional_data(serde_json::json!({"krabby_patty_formular": "secret"}))
            .build();

        let new_specimen = if i.is_multiple_of(7) {
            let s = CryopreservedSuspensionCreation::builder()
                .inner(inner)
                .build();

            SpecimenCreation::CryopreservedSuspension(s)
        } else if i.is_multiple_of(6) {
            let s = FrozenSuspensionCreation::builder().inner(inner).build();

            SpecimenCreation::FrozenSuspension(s)
        } else if i.is_multiple_of(5) {
            let s = CryopreservedTissueCreation::builder().inner(inner).build();

            SpecimenCreation::CryopreservedTissue(s)
        } else if i.is_multiple_of(4) {
            let s = FixedTissueCreation::builder()
                .inner(inner)
                .fixative(TissueFixative::VARIANTS.choose_unwrap())
                .build();

            SpecimenCreation::FixedTissue(s)
        } else if i.is_multiple_of(3) {
            let s = FrozenTissueCreation::builder().inner(inner).build();

            SpecimenCreation::FrozenTissue(s)
        } else if i.is_multiple_of(2) {
            let s = FixedBlockCreation::builder()
                .inner(inner)
                .fixative(BlockFixative::VARIANTS.choose_unwrap())
                .embedded_in(FixedBlockEmbeddingMatrix::VARIANTS.choose_unwrap())
                .build();

            SpecimenCreation::FixedBlock(s)
        } else {
            let s = FrozenBlockCreation::builder()
                .inner(inner)
                .embedded_in(FrozenBlockEmbeddingMatrix::VARIANTS.choose_unwrap())
                .fixative(BlockFixative::VARIANTS.choose_unwrap())
                .build();

            SpecimenCreation::FrozenBlock(s)
        };

        db_conn
            .interact(|db_conn| {
                new_specimen.execute(db_conn).unwrap();
            })
            .await
            .unwrap();
    }

    async fn insert_suspensions(&'static self) {
        let specimens = self
            .all_extract::<SpecimenQuery, _, _, _>(|s| (s.id(), s.submitted_by()))
            .await;

        let join_set: JoinSet<_> = specimens
            .into_iter()
            .map(|(id, submitter)| self.insert_random_suspension(id, submitter))
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_suspension(&self, specimen_id: Uuid, preparer_id: Uuid) {
        let new_suspension = SuspensionCreation::builder()
            .inner(
                SuspensionFields::builder()
                    .readable_id(random_non_empty_string())
                    .parent_specimen_id(specimen_id)
                    .build(),
            )
            .target_cell_recovery(RangedU32::new(10_000).unwrap())
            .preparer_ids(preparer_id)
            .build();
        let new_suspension = (new_suspension, SuspensionContent::VARIANTS.choose_unwrap());

        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(|db_conn| new_suspension.execute(db_conn).unwrap())
            .await
            .unwrap();
    }

    async fn insert_suspension_pools(&'static self) {
        let mut suspension_ids = self
            .all_extract::<SuspensionQuery, _, _, _>(SuspensionSummary::id)
            .await;
        let mut multiplexing_tags = self.all_extract::<(), _, _, _>(MultiplexingTag::id).await;
        let people_ids = self.all_people_ids().await;

        let mut join_set = JoinSet::new();
        for _ in 0..N_SUSPENSION_POOLS {
            let mut suspension_tags = Vec::with_capacity(N_SUSPENSIONS_PER_POOL);
            for _ in 0..N_SUSPENSIONS_PER_POOL {
                let suspension_tag = SuspensionTagging::builder()
                    .suspension_id(suspension_ids.swap_remove(0))
                    .tag_id(multiplexing_tags.swap_remove(0))
                    .build();

                suspension_tags.push(suspension_tag);
            }
            join_set.spawn(
                self.insert_random_suspension_pool(suspension_tags, people_ids.choose_unwrap()),
            );
        }

        join_set.join_all().await;
    }

    async fn insert_random_suspension_pool(
        &self,
        suspensions: Vec<SuspensionTagging>,
        preparer_id: Uuid,
    ) {
        let suspension_pool = SuspensionPoolCreation {
            inner: SuspensionPoolFields::builder()
                .name(random_non_empty_string())
                .readable_id(random_non_empty_string())
                .pooled_at(random_time())
                .build(),
            preparer_ids: preparer_id.into(),
            suspensions: NonEmptyVec::new(suspensions).unwrap(),
        };

        let db_conn = self.root_db_conn().await;
        db_conn
            .interact(|db_conn| suspension_pool.execute(db_conn).unwrap())
            .await
            .unwrap();
    }

    async fn insert_singleplex_chromium_runs(&'static self) {
        let mut suspension_ids = self
            .all_extract::<SuspensionQuery, _, _, _>(SuspensionSummary::id)
            .await;
        let people_ids = self.all_people_ids().await;
        let three_prime_gex_query = TenxAssayQuery::builder()
            .filter(
                TenxAssayFilter::builder()
                    .names(["Universal 3' Gene Expression".to_owned()])
                    .sample_multiplexing([SampleMultiplexing::Singleplex])
                    .chemistry_versions(["v4 - GEM-X".to_owned()])
                    .library_types([vec![LibraryType::GeneExpression]])
                    .build(),
            )
            .build();

        let db_conn = self.root_db_conn().await;

        let three_prime_gex_assay_id = db_conn
            .interact(|db_conn| three_prime_gex_query.execute(db_conn).unwrap())
            .await
            .unwrap();
        assert_eq!(three_prime_gex_assay_id.len(), 1);
        let three_prime_gex_assay_id = three_prime_gex_assay_id[0].id();

        let mut join_set = JoinSet::new();
        for _ in 0..N_SINGLEPLEX_CHROMIUM_RUNS {
            let this_run_suspensions = (0..MAX_GEM_POOLS_PER_NON_OCM_RUN)
                .map(|_| suspension_ids.swap_remove(0))
                .collect();

            join_set.spawn(self.insert_random_singleplex_chromium_run(
                three_prime_gex_assay_id,
                people_ids.choose_unwrap(),
                this_run_suspensions,
            ));
        }

        join_set.join_all().await;
    }

    async fn insert_random_singleplex_chromium_run(
        &self,
        assay_id: Uuid,
        run_by: Uuid,
        suspension_ids: Vec<Uuid>,
    ) {
        let chromium_run = ChromiumRunCreation::Singleplex {
            inner: random_chromium_run_fields(assay_id, run_by),
            gem_pools: NonEmptyVec::new(
                suspension_ids
                    .into_iter()
                    .map(|suspension_id| SingleplexGemPool {
                        inner: random_gem_pool_fields(),
                        loading: SingleplexChipLoading::builder()
                            .inner(random_chip_loading_fields())
                            .suspension_id(suspension_id)
                            .build(),
                    })
                    .collect(),
            )
            .unwrap(),
        };

        let db_conn = self.root_db_conn().await;
        db_conn
            .interact(|db_conn| chromium_run.execute(db_conn).unwrap())
            .await
            .unwrap();
    }

    async fn insert_ocm_chromium_runs(&'static self) {
        let mut suspension_ids = self
            .all_extract::<SuspensionQuery, _, _, _>(SuspensionSummary::id)
            .await;
        let people_ids = self.all_people_ids().await;
        let ocm_gex_query = TenxAssayQuery::builder()
            .filter(
                TenxAssayFilter::builder()
                    .names(["Universal 3' Gene Expression".to_owned()])
                    .sample_multiplexing([SampleMultiplexing::OnChipMultiplexing])
                    .chemistry_versions(["v4 - GEM-X".to_owned()])
                    .library_types([vec![LibraryType::GeneExpression]])
                    .build(),
            )
            .build();

        let db_conn = self.root_db_conn().await;

        let ocm_assay_id = db_conn
            .interact(|db_conn| ocm_gex_query.execute(db_conn).unwrap())
            .await
            .unwrap();
        assert_eq!(ocm_assay_id.len(), 1);
        let ocm_assay_id = ocm_assay_id[0].id();

        let mut join_set = JoinSet::new();
        // We already used up all the suspension IDs when inserting singleplex Chromium
        // runs, so no matter what we will have to reuse suspension IDs. These OCM runs
        // will also use them all up anyways.
        for _ in 0..N_OCM_CHROMIUM_RUNS {
            let this_run_suspensions = (0..MAX_GEM_POOLS_PER_OCM_RUN)
                .map(|_| {
                    (0..MAX_SUSPENSIONS_PER_OCM_GEM_POOL)
                        .map(|_| suspension_ids.swap_remove(0))
                        .collect()
                })
                .collect();

            join_set.spawn(self.insert_random_ocm_chromium_run(
                ocm_assay_id,
                people_ids.choose_unwrap(),
                this_run_suspensions,
            ));
        }

        join_set.join_all().await;
    }

    async fn insert_random_ocm_chromium_run(
        &self,
        assay_id: Uuid,
        run_by: Uuid,
        suspension_ids: Vec<Vec<Uuid>>,
    ) {
        let mut gem_pools = Vec::with_capacity(MAX_GEM_POOLS_PER_OCM_RUN);
        for suspension_id_group in suspension_ids {
            let loadings = suspension_id_group
                .into_iter()
                .enumerate()
                .map(|(j, id)| {
                    OcmChipLoading::builder()
                        .inner(random_chip_loading_fields())
                        .suspension_id(id)
                        .ocm_barcode_id(OcmBarcodeId::VARIANTS[j])
                        .build()
                })
                .collect();

            gem_pools.push(OcmGemPool {
                inner: random_gem_pool_fields(),
                loading: NonEmptyVec::new(loadings).unwrap(),
            });
        }

        let chromium_run = ChromiumRunCreation::OnChipMultiplexing {
            inner: random_chromium_run_fields(assay_id, run_by),
            gem_pools: NonEmptyVec::new(gem_pools).unwrap(),
        };

        let db_conn = self.root_db_conn().await;
        db_conn
            .interact(|db_conn| chromium_run.execute(db_conn).unwrap())
            .await
            .unwrap();
    }

    async fn insert_pool_multiplex_chromium_runs(&'static self) {
        let mut suspension_pool_ids = self
            .all_extract::<SuspensionPoolQuery, _, _, _>(SuspensionPool::id)
            .await;
        let people_ids = self.all_people_ids().await;
        let flex_query = TenxAssayQuery::builder()
            .filter(
                TenxAssayFilter::builder()
                    .names(["Flex Gene Expression".to_owned()])
                    .sample_multiplexing([SampleMultiplexing::FlexBarcode])
                    .chemistry_versions(["v1 - GEM-X".to_owned()])
                    .library_types([vec![LibraryType::GeneExpression]])
                    .build(),
            )
            .build();

        let db_conn = self.root_db_conn().await;

        let flex_assay_id = db_conn
            .interact(|db_conn| flex_query.execute(db_conn).unwrap())
            .await
            .unwrap();
        assert_eq!(flex_assay_id.len(), 1);
        let flex_assay_id = flex_assay_id[0].id();

        let mut join_set = JoinSet::new();
        for _ in 0..N_POOL_MULTIPLEX_CHROMIUM_RUNS {
            let this_run_suspension_pool_ids = (0..MAX_GEM_POOLS_PER_NON_OCM_RUN)
                .map(|_| suspension_pool_ids.swap_remove(0))
                .collect();

            join_set.spawn(self.insert_random_pool_multiplex_chromium_run(
                flex_assay_id,
                people_ids.choose_unwrap(),
                this_run_suspension_pool_ids,
            ));
        }

        join_set.join_all().await;
    }

    async fn insert_random_pool_multiplex_chromium_run(
        &self,
        assay_id: Uuid,
        run_by: Uuid,
        suspension_pool_ids: Vec<Uuid>,
    ) {
        let chromium_run = ChromiumRunCreation::PoolMultiplex {
            inner: random_chromium_run_fields(assay_id, run_by),
            gem_pools: NonEmptyVec::new(
                suspension_pool_ids
                    .into_iter()
                    .map(|pool_id| PoolMultiplexGemPool {
                        inner: random_gem_pool_fields(),
                        loading: PoolMultiplexChipLoading::builder()
                            .inner(random_chip_loading_fields())
                            .suspension_pool_id(pool_id)
                            .build(),
                    })
                    .collect(),
            )
            .unwrap(),
        };

        let db_conn = self.root_db_conn().await;
        db_conn
            .interact(|db_conn| chromium_run.execute(db_conn).unwrap())
            .await
            .unwrap();
    }

    async fn insert_cdna(&'static self) {
        let gem_pool_ids = self
            .all_extract::<GemPoolQuery, _, _, _>(GemPoolSummary::id)
            .await;
        let people_ids = self.all_people_ids().await;

        let join_set: JoinSet<_> = gem_pool_ids
            .into_iter()
            .map(|id| self.insert_random_cdna(id, people_ids.choose_unwrap()))
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_cdna(&self, gem_pool_id: Uuid, preparer_id: Uuid) {
        let cdna = CdnaCreation::builder()
            .inner(
                CdnaFields::builder()
                    .gem_pool_id(gem_pool_id)
                    .library_type(LibraryType::GeneExpression)
                    .readable_id(random_non_empty_string())
                    .build(),
            )
            .n_amplification_cycles(random_u8())
            .prepared_at(random_time())
            .preparer_ids(preparer_id)
            .volume_µl(random_u8())
            .build();

        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(|db_conn| cdna.execute(db_conn).unwrap())
            .await
            .unwrap();
    }

    async fn insert_libraries(&'static self) {
        let cdna_ids = self
            .all_extract::<CdnaQuery, _, _, _>(CdnaSummary::id)
            .await;
        let people_ids = self.all_people_ids().await;

        let join_set: JoinSet<_> = cdna_ids
            .into_iter()
            .map(|id| self.insert_random_library(id, people_ids.choose_unwrap()))
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_library(&self, cdna_id: Uuid, preparer_id: Uuid) {
        // Technically this isn't 100% correct because Flex libraries and Universal 3'
        // GEX libraries have different index sets and volumes, but we don't care here
        let library = LibraryCreation::builder()
            .inner(
                LibraryFields::builder()
                    .cdna_id(cdna_id)
                    .dual_index_set_name(NonEmptyString::new("SI-TT-A1").unwrap())
                    .readable_id(random_non_empty_string())
                    .build(),
            )
            .number_of_sample_index_pcr_cycles(RangedU16::new(10).unwrap())
            .prepared_at(random_time())
            .preparer_ids(preparer_id)
            .volume_µl(35)
            .target_reads_per_cell(RangedU32::new(50_000).unwrap())
            .build();

        let db_conn = self.root_db_conn().await;
        db_conn
            .interact(|db_conn| library.execute(db_conn).unwrap())
            .await
            .unwrap();
    }

    async fn insert_chromium_datasets(&'static self) {
        let library_ids = self
            .all_extract::<LibraryQuery, _, _, _>(LibrarySummary::id)
            .await;
        let lab_ids = self.all_extract::<LabQuery, _, _, _>(LabSummary::id).await;

        let join_set: JoinSet<_> = library_ids
            .into_iter()
            .map(|id| self.insert_random_chromium_dataset(id, lab_ids.choose_unwrap()))
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_chromium_dataset(&self, library_id: Uuid, lab_id: Uuid) {
        // It's easier to construct this as JSON
        let dataset = json!(
            {
                "name": random_non_empty_string(),
                "lab_id": lab_id,
                "data_path": random_non_empty_string(),
                "delivered_at": random_time(),
                "library_ids": vec![library_id],
                "cmdline": "cellranger multi"
            }
        );
        let dataset: ChromiumDatasetCreation = serde_json::from_value(dataset).unwrap();

        let db_conn = self.root_db_conn().await;
        db_conn
            .interact(|db_conn| {
                use cellnoor_schema::{chromium_dataset_web_summaries as ws, chromium_dataset_metrics_files as mf};
                let created_ds_id = dataset.execute(db_conn).unwrap().id();

                let values = |i| {
                    let content = format!("<!DOCTYPE html><html><head><title>Web summary</title></head><body>web summary{i} - {created_ds_id}</body></html>");
                    (ws::dataset_id.eq(created_ds_id), ws::directory.eq(format!("specimen{i}")), ws::filename.eq("web_summary.html"), ws::content.eq(content.into_bytes()))
                };
                diesel::insert_into(ws::table).values([values(0), values(1)]).execute(db_conn).unwrap();

                let values = |i| {
                    let raw_content = format!("ds_id, some_metric,another_metric,n\n{created_ds_id}100,42,{i}");
                    let parsed_data = serde_json::json!({"ds_id": created_ds_id, "some_metric": 100, "another_metric": 42, "n": i});
                    (mf::dataset_id.eq(created_ds_id), mf::directory.eq(format!("specimen{i}")), mf::filename.eq("metrics_summary.csv"), mf::raw_content.eq(raw_content.into_bytes()), mf::content_type.eq("text/csv"), mf::parsed_data.eq(parsed_data))
                };
                diesel::insert_into(mf::table).values([values(0), values(1)]).execute(db_conn).unwrap();
            })
            .await
            .unwrap();
    }

    async fn root_db_conn(&self) -> Connection {
        self.root_db_pool.get().await.unwrap()
    }

    async fn all<Q, T>(&self) -> Vec<T>
    where
        Q: DefaultWithNoLimit + db::Operation<Vec<T>>,
        T: 'static + Send,
    {
        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(|db_conn| Q::default_with_no_limit().execute(db_conn).unwrap())
            .await
            .unwrap()
    }

    async fn all_extract<Q, F, T, U>(&self, f: F) -> Vec<U>
    where
        Q: DefaultWithNoLimit + db::Operation<Vec<T>>,
        T: 'static + Send,
        F: Fn(&T) -> U,
    {
        self.all::<Q, _>().await.iter().map(f).collect()
    }

    async fn all_people_ids(&'static self) -> &'static [Uuid] {
        PEOPLE_IDS
            .get_or_init(|| self.all_extract::<PersonQuery, _, _, _>(PersonSummary::id))
            .await
    }
}

pub trait DefaultWithNoLimit {
    fn default_with_no_limit() -> Self;
}

impl<F, O> DefaultWithNoLimit for generic_query::Query<F, O>
where
    O: Default,
{
    fn default_with_no_limit() -> Self {
        Self::default_with_no_limit()
    }
}

impl DefaultWithNoLimit for () {
    fn default_with_no_limit() -> Self {}
}

impl<U, F, O> DefaultWithNoLimit for (U, generic_query::Query<F, O>)
where
    U: From<Uuid>,
    O: Default,
{
    fn default_with_no_limit() -> Self {
        (
            Uuid::default().into(),
            generic_query::Query::<F, O>::default_with_no_limit(),
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Database {
    pub institutions: Vec<Institution>,
    pub people: Vec<PersonSummary>,
    pub labs: Vec<LabSummary>,
    pub specimens: Vec<SpecimenSummary>,
    pub _suspensions: Vec<SuspensionSummary>,
    pub suspension_pools: Vec<SuspensionPool>,
    pub _gem_pools: Vec<GemPoolSummary>,
    pub _cdna: Vec<CdnaSummary>,
    pub _libraries: Vec<LibrarySummary>,
    pub _chromium_datasets: Vec<ChromiumDatasetSummary>,
}

impl Database {
    async fn new(test_state: &'static TestState) -> Self {
        test_state.populate_db().await;

        let (
            institutions,
            people,
            labs,
            specimens,
            suspensions,
            suspension_pools,
            gem_pools,
            cdna,
            libraries,
            chromium_datasets,
        ) = tokio::join!(
            test_state.all::<InstitutionQuery, _>(),
            test_state.all::<PersonQuery, _>(),
            test_state.all::<LabQuery, _>(),
            test_state.all::<SpecimenQuery, _>(),
            test_state.all::<SuspensionQuery, _>(),
            test_state.all::<SuspensionPoolQuery, _>(),
            test_state.all::<GemPoolQuery, _>(),
            test_state.all::<CdnaQuery, _>(),
            test_state.all::<LibraryQuery, _>(),
            test_state.all::<ChromiumDatasetQuery, _>()
        );

        Self {
            institutions,
            people,
            labs,
            specimens,
            _suspensions: suspensions,
            suspension_pools,
            _gem_pools: gem_pools,
            _cdna: cdna,
            _libraries: libraries,
            _chromium_datasets: chromium_datasets,
        }
    }
}

fn random_string() -> String {
    let mut rng = rand::rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

fn random_non_empty_string() -> NonEmptyString {
    NonEmptyString::new(random_string()).unwrap()
}

// These numbers correspond to the first second of the year -4000 and the last second of the year 4000 (https://www.postgresql.org/docs/current/datatype-datetime.html)
const TIME: Range<i64> = -188_395_009_438..64_092_229_199;

fn random_time() -> Timestamp {
    let mut rng = rand::rng();
    Timestamp::from_second(TIME.choose(&mut rng).unwrap()).unwrap()
}

fn random_u8() -> u8 {
    let mut rng = rand::rng();
    (u8::MIN..u8::MAX).choose(&mut rng).unwrap()
}

fn random_chromium_run_fields(assay_id: Uuid, run_by: Uuid) -> ChromiumRunFields {
    ChromiumRunFields::builder()
        .readable_id(random_non_empty_string())
        .assay_id(assay_id)
        .run_at(random_time())
        .run_by(run_by)
        .succeeded(true)
        .build()
}

fn random_gem_pool_fields() -> GemPoolFields {
    GemPoolFields::builder()
        .readable_id(random_non_empty_string())
        .build()
}

fn random_chip_loading_fields() -> ChipLoadingFields {
    ChipLoadingFields::builder()
        .suspension_volume_loaded(Volume::new(0))
        .buffer_volume_loaded(Volume::new(0))
        .build()
}

const N_INSTITUTIONS: usize = 4;
const N_PEOPLE_PER_INSTITUTION: usize = 32;
const N_LABS: usize = N_INSTITUTIONS * 4;

const N_SPECIMENS_PER_PERSON: usize = 2;
const N_SPECIMENS: usize = N_SPECIMENS_PER_PERSON * N_PEOPLE_PER_INSTITUTION * N_INSTITUTIONS;

const N_SUSPENSIONS: usize = N_SPECIMENS;

const N_SUSPENSION_POOLS: usize = N_SUSPENSIONS / 4;
pub const N_SUSPENSIONS_PER_POOL: usize = 2;

const N_SINGLEPLEX_CHROMIUM_RUNS: usize = N_SUSPENSIONS / MAX_GEM_POOLS_PER_NON_OCM_RUN;

const N_OCM_CHROMIUM_RUNS: usize =
    N_SUSPENSIONS / (MAX_GEM_POOLS_PER_OCM_RUN * MAX_SUSPENSIONS_PER_OCM_GEM_POOL);

const N_POOL_MULTIPLEX_CHROMIUM_RUNS: usize = N_SUSPENSION_POOLS / MAX_GEM_POOLS_PER_NON_OCM_RUN;

static PEOPLE_IDS: OnceCell<Vec<Uuid>> = OnceCell::const_new();

trait ChooseUnwrap<T> {
    fn choose_unwrap(&self) -> T;
}

impl<T> ChooseUnwrap<T> for [T]
where
    T: Copy,
{
    fn choose_unwrap(&self) -> T {
        let mut rng = rand::rng();
        *self.choose(&mut rng).unwrap()
    }
}
