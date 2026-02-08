use std::ops::Range;

use cellnoor_models::{
    cdna::{CdnaFields, CdnaQuery, CdnaSummary, NewCdna},
    chromium_dataset::{ChromiumDatasetQuery, ChromiumDatasetSummary, NewChromiumDataset},
    chromium_run::{
        ChipLoadingFields, ChromiumRunFields, ChromiumRunSummary, GemPoolFields, GemPoolQuery,
        GemPoolSummary, MAX_GEM_POOLS_PER_NON_OCM_RUN, MAX_GEM_POOLS_PER_OCM_RUN,
        MAX_SUSPENSIONS_PER_OCM_GEM_POOL, NewChromiumRun, OcmBarcodeId, OcmChipLoading, OcmGemPool,
        PoolMultiplexChipLoading, PoolMultiplexGemPool, SingleplexChipLoading, SingleplexGemPool,
        Volume,
    },
    generic_query::{self, Query},
    institution::{Institution, InstitutionQuery, NewInstitution},
    library::{LibraryFields, LibraryQuery, LibrarySummary, NewLibrary},
    multiplexing_tag::MultiplexingTag,
    person::{NewPerson, PersonFields, PersonQuery, PersonSummary},
    project::{NewProject, Project, ProjectFields, ProjectQuery},
    specimen::{
        BlockFixative, Fixative, NewBlock, NewSpecimen, NewSuspensionSpecimen, NewTissue, Species,
        SpecimenCommonFields, SpecimenQuery, SpecimenSummary, SuspensionThermalPreservation,
        ThermalPreservationMethod,
    },
    suspension::{
        self, NewSuspension, NewSuspensionCommonFields, SuspensionFields, SuspensionQuery,
        SuspensionSummary,
    },
    suspension_pool::{
        NewSuspensionPool, SuspensionPool, SuspensionPoolFields, SuspensionPoolQuery,
        SuspensionTagging,
    },
    tenx_assay::{LibraryType, SampleMultiplexing, TenxAssay, TenxAssayFilter, TenxAssayQuery},
};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::FutureExt;
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
    api::routes::{
        cdna::{create::insert_cdna_and_preparers, index::select_cdna},
        chromium_datasets::{
            create::insert_chromium_dataset_and_libraries, index::select_chromium_datasets,
        },
        chromium_runs::{
            create::insert_chromium_run_and_associated_data, index::select_chromium_runs,
        },
        gem_pools::index::select_gem_pools,
        institutions::{create::insert_institution, index::select_institutions},
        libraries::{create::insert_library_and_preparers, index::select_libraries},
        multiplexing_tags::index::select_multiplexing_tags,
        people::{create::insert_person, index::select_people},
        projects::{create::insert_project, index::select_projects},
        specimens::{create::insert_specimen, index::select_specimens},
        suspension_pools::{
            create::{create_suspension_pool, insert_suspension_pool_and_preparers_and_tags},
            index::select_suspension_pools,
        },
        suspensions::{create::insert_suspension, index::select_suspensions},
        tenx_assays::index::select_tenx_assays,
    },
    config::Config,
    db::{self, DbConnection, DbConnectionPool},
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
pub async fn root_db_conn() -> DbConnection {
    let state = TEST_STATE.get_or_init(TestState::new).await;
    state.root_db_conn().await
}

pub struct TestState {
    _inner: AppState,
    root_db_pool: DbConnectionPool,
}

impl TestState {
    async fn new() -> Self {
        let config = Config::read()
            .expect("test configuration should be readable from environment variables");
        let db_root_url = config.db_root_url();

        Self {
            _inner: AppState::initialize(config)
                .await
                .expect("should be able to initialize app state"),
            root_db_pool: create_test_db_pool(&db_root_url).unwrap(),
        }
    }

    async fn populate_db(&'static self) {
        // This is a safeguard so that a failure to initialize test state doesn't cause
        // endless repetition
        let institution_ids = self.all_extract(select_institutions, Institution::id).await;
        if institution_ids.len() > 1 {
            return;
        }

        let db_conn = &self.root_db_conn().await;

        self.insert_institutions(db_conn).await;
        self.insert_people(db_conn).await;
        self.insert_projects(db_conn).await;
        self.insert_specimens(db_conn).await;
        self.insert_suspensions(db_conn).await;
        self.insert_suspension_pools(db_conn).await;
        self.insert_singleplex_chromium_runs(db_conn).await;
        self.insert_ocm_chromium_runs(db_conn).await;
        self.insert_pool_multiplex_chromium_runs(db_conn).await;
        self.insert_cdna(db_conn).await;
        self.insert_libraries(db_conn).await;
        self.insert_chromium_datasets(db_conn).await;
    }

    async fn insert_institutions(&'static self, db_conn: &AsyncPgConnection) {
        let mut insertions = Vec::with_capacity(N_INSTITUTIONS);
        for _ in 0..N_INSTITUTIONS {
            let institution = NewInstitution::new(Uuid::now_v7(), random_non_empty_string());
            insertions.push(insert_institution(institution, db_conn).map(Result::unwrap));
        }

        futures::future::join_all(insertions).await;
    }

    async fn insert_people(&'static self, db_conn: &AsyncPgConnection) {
        let institution_ids = self.all_extract(select_institutions, Institution::id).await;

        let mut insertions = Vec::with_capacity(N_PEOPLE);

        // Skip the first institution since that already has a person
        for inst in &institution_ids[1..] {
            for _ in 0..N_PEOPLE_PER_INSTITUTION {
                let name = random_string();
                let email = format!("{name}@example.com");
                let person = NewPerson::builder()
                    .inner(
                        PersonFields::builder()
                            .name(NonEmptyString::new(name).unwrap())
                            .institution_id(*inst)
                            .build(),
                    )
                    .email(NonEmptyString::new(email).unwrap())
                    .build();

                insertions.push(insert_person(person, db_conn));
            }
        }

        futures::future::try_join_all(insertions).await.unwrap();
    }

    async fn insert_projects(&'static self, db_conn: &AsyncPgConnection) {
        const ONE_YEAR: i64 = 60 * 60 * 24 * 365;
        let mut insertions = Vec::with_capacity(N_PROJECTS);
        for _ in 0..N_PROJECTS {
            let started_at = random_time();
            let project = NewProject::builder()
                .inner(
                    ProjectFields::builder()
                        .name(random_non_empty_string())
                        .build(),
                )
                .started_at(started_at)
                .ended_at(started_at + jiff::SignedDuration::new(ONE_YEAR, 0))
                .build();

            insertions.push(insert_project(project, db_conn));
        }

        futures::future::try_join_all(insertions).await.unwrap();
    }

    async fn insert_specimens(&'static self, db_conn: &AsyncPgConnection) {
        let people_ids = self.all_people_ids();
        let project_ids = self.all_extract(select_projects, Project::id);

        let (people_ids, project_ids) = tokio::join!(people_ids, project_ids);

        let mut insertions = Vec::with_capacity(N_SPECIMENS);
        let mut counter = 0;

        for project_id in &project_ids {
            for submitted_by in &people_ids[1..] {
                counter += 1;
                insertions.push(self.insert_random_specimen(
                    counter,
                    *submitted_by,
                    *project_id,
                    db_conn,
                ));
            }
        }

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_specimen(
        &self,
        i: usize,
        submitted_by: Uuid,
        project_id: Uuid,
        db_conn: &AsyncPgConnection,
    ) {
        let inner = SpecimenCommonFields::builder()
            .readable_id(random_non_empty_string())
            .name(random_non_empty_string())
            .submitted_by(submitted_by)
            .project_id(project_id)
            .received_at(random_time())
            .species(Species::VARIANTS.choose_unwrap())
            .tissue(random_non_empty_string())
            .additional_data(serde_json::json!({"krabby_patty_formular":
    "secret"}))
            .build();

        let new_specimen = if i.is_multiple_of(9) {
            NewSpecimen::Block(NewBlock::CarboxymethylCellulose {
                inner,
                fixative: None,
            })
        } else if i.is_multiple_of(8) {
            NewSpecimen::Block(NewBlock::OptimalCuttingTemperatureCompound {
                inner,
                fixative: None,
            })
        } else if i.is_multiple_of(7) {
            NewSpecimen::Block(NewBlock::Paraffin {
                inner,
                fixative: BlockFixative::VARIANTS.choose_unwrap(),
            })
        } else if i.is_multiple_of(6) {
            NewSpecimen::Suspension(NewSuspensionSpecimen::Fresh { inner })
        } else if i.is_multiple_of(5) {
            NewSpecimen::Suspension(NewSuspensionSpecimen::Fixed {
                inner,
                fixative: Fixative::VARIANTS.choose_unwrap(),
            })
        } else if i.is_multiple_of(4) {
            NewSpecimen::Suspension(NewSuspensionSpecimen::ThermallyPreserved {
                inner,
                thermal_preservation_method: SuspensionThermalPreservation::VARIANTS
                    .choose_unwrap(),
            })
        } else if i.is_multiple_of(3) {
            NewSpecimen::Tissue(NewTissue::Fresh { inner })
        } else if i.is_multiple_of(2) {
            NewSpecimen::Tissue(NewTissue::Fixed {
                inner,
                fixative: Fixative::VARIANTS.choose_unwrap(),
            })
        } else {
            NewSpecimen::Tissue(NewTissue::ThermallyPreserved {
                inner,
                thermal_preservation_method: ThermalPreservationMethod::VARIANTS.choose_unwrap(),
            })
        };

        insert_specimen(new_specimen, db_conn).await.unwrap();
    }

    async fn insert_suspensions(&'static self, db_conn: &AsyncPgConnection) {
        let specimens = self.all(select_specimens).await;

        let insertions = specimens
            .iter()
            .map(|s| self.insert_random_suspension(s, db_conn));

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_suspension(
        &self,
        specimen: &SpecimenSummary,
        db_conn: &AsyncPgConnection,
    ) {
        let new_suspension = {
            let common = NewSuspensionCommonFields::builder()
                .inner(
                    SuspensionFields::builder()
                        .readable_id(random_non_empty_string())
                        .parent_specimen_id(specimen.id())
                        .build(),
                )
                .target_cell_recovery(RangedU32::new(10_000).unwrap())
                .preparer_ids(specimen.submitted_by())
                .build();

            NewSuspension::Cell(common)
        };

        insert_suspension(specimen.project_id(), new_suspension, db_conn)
            .await
            .unwrap();
    }

    async fn insert_suspension_pools(&'static self, db_conn: &AsyncPgConnection) {
        let suspensions = self.all(select_suspensions);
        let multiplexing_tags = select_multiplexing_tags(db_conn)
            .map(Result::unwrap)
            .map(Vec::into_iter)
            .map(|tags| tags.map(|t| t.id()).collect());
        let people_ids = self.all_people_ids();

        let (mut suspensions, mut multiplexing_tags, people_ids): (_, Vec<_>, _) =
            tokio::join!(suspensions, multiplexing_tags, people_ids);

        let mut insertions = Vec::with_capacity(N_SUSPENSION_POOLS);
        for _ in 0..N_SUSPENSION_POOLS {
            let project_id = suspensions[0].project_id();
            let mut suspension_tags = Vec::with_capacity(N_SUSPENSIONS_PER_POOL);
            for _ in 0..N_SUSPENSIONS_PER_POOL {
                let suspension_tag = SuspensionTagging::builder()
                    .suspension_id(suspensions.swap_remove(0).id())
                    .tag_id(multiplexing_tags.swap_remove(0))
                    .build();

                suspension_tags.push(suspension_tag);
            }

            insertions.push(self.insert_random_suspension_pool(
                project_id,
                suspension_tags,
                people_ids.choose_unwrap(),
                db_conn,
            ));
        }

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_suspension_pool(
        &self,
        project_id: Uuid,
        suspensions: Vec<SuspensionTagging>,
        preparer_id: Uuid,
        db_conn: &AsyncPgConnection,
    ) {
        let suspension_pool = SuspensionPoolFields::builder()
            .name(random_non_empty_string())
            .readable_id(random_non_empty_string())
            .pooled_at(random_time())
            .build();

        let preparer_ids = preparer_id.into();

        let pooled_suspensions = NonEmptyVec::new(suspensions).unwrap();

        insert_suspension_pool_and_preparers_and_tags(
            project_id,
            suspension_pool,
            preparer_ids,
            pooled_suspensions,
            db_conn,
        )
        .await
        .unwrap();
    }

    async fn insert_singleplex_chromium_runs(&'static self, db_conn: &AsyncPgConnection) {
        let suspensions = self.all(select_suspensions);
        let people = self.all_people_ids();
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
        let three_prime_gex_assays =
            select_tenx_assays(three_prime_gex_query, db_conn).map(Result::unwrap);

        let (mut suspensions, people, three_prime_gex_assay_id) =
            tokio::join!(suspensions, people, three_prime_gex_assays);

        assert_eq!(three_prime_gex_assay_id.len(), 1);
        let three_prime_gex_assay_id = three_prime_gex_assay_id[0].id();

        let mut insertions = Vec::with_capacity(N_SINGLEPLEX_CHROMIUM_RUNS);
        for _ in 0..N_SINGLEPLEX_CHROMIUM_RUNS {
            let this_run_suspensions = (0..MAX_GEM_POOLS_PER_NON_OCM_RUN)
                .map(|_| suspensions.swap_remove(0))
                .collect();

            insertions.push(self.insert_random_singleplex_chromium_run(
                three_prime_gex_assay_id,
                people.choose_unwrap(),
                this_run_suspensions,
                db_conn,
            ));
        }

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_singleplex_chromium_run(
        &self,
        assay_id: Uuid,
        run_by: Uuid,
        suspensions: Vec<SuspensionSummary>,
        db_conn: &AsyncPgConnection,
    ) {
        let project_id = suspensions[0].project_id();
        let chromium_run = NewChromiumRun::Singleplex {
            inner: random_chromium_run_fields(assay_id, run_by),
            gem_pools: NonEmptyVec::new(
                suspensions
                    .iter()
                    .map(SuspensionSummary::id)
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

        insert_chromium_run_and_associated_data(project_id, chromium_run, db_conn)
            .await
            .unwrap();
    }

    async fn insert_ocm_chromium_runs(&'static self, db_conn: &AsyncPgConnection) {
        let suspensions = self.all(select_suspensions);
        let people_ids = self.all_people_ids();
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

        let ocm_assays = select_tenx_assays(ocm_gex_query, db_conn).map(Result::unwrap);

        let (mut suspensions, people_ids, ocm_assay_id) =
            tokio::join!(suspensions, people_ids, ocm_assays);

        assert_eq!(ocm_assay_id.len(), 1);
        let ocm_assay_id = ocm_assay_id[0].id();

        let mut insertions = Vec::with_capacity(N_OCM_CHROMIUM_RUNS);
        // We already used up all the suspension IDs when inserting singleplex
        // Chromium runs, so no matter what we will have to reuse suspension
        // IDs. These OCM runs will also use them all up anyways.
        for _ in 0..N_OCM_CHROMIUM_RUNS {
            let this_run_suspensions = (0..MAX_GEM_POOLS_PER_OCM_RUN)
                .map(|_| {
                    (0..MAX_SUSPENSIONS_PER_OCM_GEM_POOL)
                        .map(|_| suspensions.swap_remove(0))
                        .collect()
                })
                .collect();

            insertions.push(self.insert_random_ocm_chromium_run(
                ocm_assay_id,
                people_ids.choose_unwrap(),
                this_run_suspensions,
                db_conn,
            ));
        }

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_ocm_chromium_run(
        &self,
        assay_id: Uuid,
        run_by: Uuid,
        suspensions: Vec<Vec<SuspensionSummary>>,
        db_conn: &AsyncPgConnection,
    ) {
        let project_id = suspensions[0][0].project_id();
        let mut gem_pools = Vec::with_capacity(MAX_GEM_POOLS_PER_OCM_RUN);
        for suspension_group in suspensions {
            let loadings = suspension_group
                .iter()
                .map(SuspensionSummary::id)
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

        let chromium_run = NewChromiumRun::OnChipMultiplexing {
            inner: random_chromium_run_fields(assay_id, run_by),
            gem_pools: NonEmptyVec::new(gem_pools).unwrap(),
        };

        insert_chromium_run_and_associated_data(project_id, chromium_run, db_conn)
            .await
            .unwrap();
    }

    async fn insert_pool_multiplex_chromium_runs(&'static self, db_conn: &AsyncPgConnection) {
        let suspension_pools = self.all(select_suspension_pools);
        let people_ids = self.all_people_ids();
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

        let flex_assays = select_tenx_assays(flex_query, db_conn).map(Result::unwrap);

        let (mut suspension_pools, people_ids, flex_assays) =
            tokio::join!(suspension_pools, people_ids, flex_assays);

        assert_eq!(flex_assays.len(), 1);
        let flex_assay_id = flex_assays[0].id();

        let mut insertions = Vec::with_capacity(N_POOL_MULTIPLEX_CHROMIUM_RUNS);
        for _ in 0..N_POOL_MULTIPLEX_CHROMIUM_RUNS {
            let this_run_suspension_pools = (0..MAX_GEM_POOLS_PER_NON_OCM_RUN)
                .map(|_| suspension_pools.swap_remove(0))
                .collect();

            insertions.push(self.insert_random_pool_multiplex_chromium_run(
                flex_assay_id,
                people_ids.choose_unwrap(),
                this_run_suspension_pools,
                db_conn,
            ));
        }

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_pool_multiplex_chromium_run(
        &self,
        assay_id: Uuid,
        run_by: Uuid,
        suspension_pools: Vec<SuspensionPool>,
        db_conn: &AsyncPgConnection,
    ) {
        let project_id = suspension_pools[0].project_id();
        let chromium_run = NewChromiumRun::PoolMultiplex {
            inner: random_chromium_run_fields(assay_id, run_by),
            gem_pools: NonEmptyVec::new(
                suspension_pools
                    .iter()
                    .map(SuspensionPool::id)
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

        insert_chromium_run_and_associated_data(project_id, chromium_run, db_conn)
            .await
            .unwrap();
    }

    async fn insert_cdna(&'static self, db_conn: &AsyncPgConnection) {
        let chromium_runs = self.all(select_chromium_runs);
        let gem_pools = self.all(select_gem_pools);
        let people_ids = self.all_people_ids();

        let (chromium_runs, gem_pools, people_ids) =
            tokio::join!(chromium_runs, gem_pools, people_ids);

        let insertions = gem_pools
            .iter()
            .map(|gem_pool| {
                (
                    chromium_runs
                        .iter()
                        .find(|run| run.id() == gem_pool.chromium_run_id())
                        .map(ChromiumRunSummary::project_id)
                        .unwrap(),
                    gem_pool.id(),
                )
            })
            .map(|project_pool_pair| {
                self.insert_random_cdna(project_pool_pair, people_ids.choose_unwrap(), db_conn)
            });

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_cdna(
        &self,
        (project_id, gem_pool_id): (Uuid, Uuid),
        preparer_id: Uuid,
        db_conn: &AsyncPgConnection,
    ) {
        let cdna = NewCdna::builder()
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

        insert_cdna_and_preparers(project_id, cdna, db_conn)
            .await
            .unwrap();
    }

    async fn insert_libraries(&'static self, db_conn: &AsyncPgConnection) {
        let cdna = self.all(select_cdna);
        let people_ids = self.all_people_ids();

        let (cdna, people_ids) = tokio::join!(cdna, people_ids);

        let insertions = cdna
            .into_iter()
            .map(|cdna| self.insert_random_library(cdna, people_ids.choose_unwrap(), db_conn));

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_library(
        &self,
        cdna: CdnaSummary,
        preparer_id: Uuid,
        db_conn: &AsyncPgConnection,
    ) {
        // Technically this isn't 100% correct because Flex libraries and
        // Universal 3' GEX libraries have different index sets and volumes,
        // but we don't care here
        let library = NewLibrary::builder()
            .inner(
                LibraryFields::builder()
                    .cdna_id(cdna.id())
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

        insert_library_and_preparers(cdna.project_id(), library, db_conn)
            .await
            .unwrap();
    }

    async fn insert_chromium_datasets(&'static self, db_conn: &AsyncPgConnection) {
        let libraries = self.all(select_libraries).await;

        let insertions = libraries
            .into_iter()
            .map(|l| self.insert_random_chromium_dataset(l, db_conn));

        futures::future::join_all(insertions).await;
    }

    async fn insert_random_chromium_dataset(
        &self,
        library: LibrarySummary,
        mut db_conn: &AsyncPgConnection,
    ) {
        use cellnoor_schema::{
            chromium_dataset_metrics_files as mf, chromium_dataset_web_summaries as ws,
        };

        // It's easier to construct this as JSON
        let dataset = json!(
            {
                "name": random_non_empty_string(),
                "data_path": random_non_empty_string(),
                "delivered_at": random_time(),
                "library_ids": vec![library.id()],
                "cmdline": "cellranger multi"
            }
        );
        let dataset: NewChromiumDataset = serde_json::from_value(dataset).unwrap();

        let dataset_id =
            insert_chromium_dataset_and_libraries(library.project_id(), dataset, db_conn)
                .await
                .unwrap();

        let values = |i| {
            let content = format!(
                "<!DOCTYPE html><html><head><title>Web summary</title></head><body>web summary{i} \
                 - {dataset_id}</body></html>"
            );
            (
                ws::dataset_id.eq(dataset_id),
                ws::directory.eq(format!("specimen{i}")),
                ws::filename.eq("web_summary.html"),
                ws::content.eq(content.into_bytes()),
            )
        };

        diesel::insert_into(ws::table)
            .values([values(0), values(1)])
            .execute(&mut db_conn)
            .await
            .unwrap();

        let values = |i| {
            let raw_content =
                format!("ds_id, some_metric,another_metric,n\n{dataset_id}100,42,{i}");
            let parsed_data = serde_json::json!({"ds_id": dataset_id, "some_metric": 100, "another_metric": 42, "n": i});
            (
                mf::dataset_id.eq(dataset_id),
                mf::directory.eq(format!("specimen{i}")),
                mf::filename.eq("metrics_summary.csv"),
                mf::raw_content.eq(raw_content.into_bytes()),
                mf::content_type.eq("text/csv"),
                mf::parsed_data.eq(parsed_data),
            )
        };
        diesel::insert_into(mf::table)
            .values([values(0), values(1)])
            .execute(&mut db_conn)
            .await
            .unwrap();
    }

    async fn root_db_conn(&self) -> DbConnection {
        DbConnection::new(self.root_db_pool.get().await.unwrap())
    }

    async fn all<SelectFn, Filter, OrderBy, Return>(&self, select: SelectFn) -> Vec<Return>
    where
        Filter: Default,
        OrderBy: Default,
        SelectFn: AsyncFn(
            generic_query::Query<Filter, OrderBy>,
            &AsyncPgConnection,
        ) -> Result<Vec<Return>, db::Error>,
        Return: 'static + Send,
    {
        let db_conn = self.root_db_conn().await;
        let query = generic_query::Query::default_with_no_limit();

        select(query, &db_conn).await.unwrap()
    }

    async fn all_extract<SelectFn, ExtractFn, Filter, OrderBy, Intermediate, Return>(
        &self,
        select: SelectFn,
        extract: ExtractFn,
    ) -> Vec<Return>
    where
        Filter: Default,
        OrderBy: Default,
        SelectFn: AsyncFn(
            generic_query::Query<Filter, OrderBy>,
            &AsyncPgConnection,
        ) -> Result<Vec<Intermediate>, db::Error>,
        Intermediate: 'static + Send,
        ExtractFn: Fn(&Intermediate) -> Return,
    {
        self.all(select).await.iter().map(extract).collect()
    }

    async fn all_people_ids(&'static self) -> &'static [Uuid] {
        PEOPLE_IDS
            .get_or_init(|| self.all_extract(select_people, PersonSummary::id))
            .await
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Database {
    pub institutions: Vec<Institution>,
    pub people: Vec<PersonSummary>,
    pub projects: Vec<Project>,
    pub specimens: Vec<SpecimenSummary>,
    pub suspensions: Vec<SuspensionSummary>,
    pub suspension_pools: Vec<SuspensionPool>,
    pub gem_pools: Vec<GemPoolSummary>,
    pub cdna: Vec<CdnaSummary>,
    pub libraries: Vec<LibrarySummary>,
    pub chromium_datasets: Vec<ChromiumDatasetSummary>,
}

impl Database {
    async fn new(test_state: &'static TestState) -> Self {
        test_state.populate_db().await;

        let (
            institutions,
            people,
            projects,
            specimens,
            suspensions,
            suspension_pools,
            gem_pools,
            cdna,
            libraries,
            chromium_datasets,
        ) = tokio::join!(
            test_state.all(select_institutions),
            test_state.all(select_people),
            test_state.all(select_projects),
            test_state.all(select_specimens),
            test_state.all(select_suspensions),
            test_state.all(select_suspension_pools),
            test_state.all(select_gem_pools),
            test_state.all(select_cdna),
            test_state.all(select_libraries),
            test_state.all(select_chromium_datasets)
        );

        Self {
            institutions,
            people,
            projects,
            specimens,
            suspensions,
            suspension_pools,
            gem_pools,
            cdna,
            libraries,
            chromium_datasets,
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
const N_PEOPLE_PER_INSTITUTION: usize = 16;
const N_PEOPLE: usize = N_INSTITUTIONS * N_PEOPLE_PER_INSTITUTION;

const N_PROJECTS: usize = N_INSTITUTIONS * 2;

const N_SPECIMENS: usize = N_PEOPLE * N_PROJECTS;

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
