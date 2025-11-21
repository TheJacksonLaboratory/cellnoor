#![allow(dead_code)]
use std::ops::Range;

use deadpool_diesel::postgres::{Connection, Pool};
use jiff::Timestamp;
use non_empty_string::NonEmptyString;
use rand::{
    Rng, SeedableRng,
    distr::Alphanumeric,
    rngs::SmallRng,
    seq::{IndexedRandom, IteratorRandom},
};
use rstest::fixture;
use scamplers_models::{
    NoLimit, institution, institution::Institution, lab, person, person::Summary,
};
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
    inner: AppState,
    root_db_pool: Pool,
}

impl TestState {
    async fn new() -> Self {
        let config = Config::read()
            .expect("test configuration should be readable from environment variables");

        Self {
            inner: AppState::initialize(&config)
                .await
                .expect("should be able to initialize app state"),
            root_db_pool: create_test_db_pool(&config.db_root_url()).unwrap(),
        }
    }

    async fn populate_db(&'static self) {
        self.insert_institutions().await;
        self.insert_people().await;
        self.insert_labs().await;
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
                institution::Creation::builder()
                    .id(Uuid::now_v7())
                    .name(NonEmptyString::new(random_string()).unwrap())
                    .build()
                    .execute(db_conn)
                    .unwrap()
            })
            .await
            .unwrap();
    }

    async fn insert_people(&'static self) {
        let institution_ids = self
            .all_extract::<institution::Query, _, _, _>(Institution::id)
            .await;

        let join_set: JoinSet<_> = (0..N_PEOPLE)
            .map(|_| self.insert_random_person(institution_ids.choose_unwrap()))
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_person(&self, institution_id: Uuid) {
        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(move |db_conn| {
                let name = random_string();
                let email = format!("{name}@example.com");

                person::Creation::builder()
                    .name(NonEmptyString::new(name).unwrap())
                    .email(NonEmptyString::new(email).unwrap())
                    .institution_id(institution_id)
                    .roles([])
                    .build()
                    .execute(db_conn)
                    .unwrap();
            })
            .await
            .unwrap();
    }

    async fn insert_labs(&'static self) {
        let people_ids = self
            .all_extract::<person::Query, _, _, _>(Summary::id)
            .await;

        let join_set: JoinSet<_> = (0..N_LABS)
            .map(|_| self.insert_random_lab(people_ids.choose_unwrap()))
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_lab(&self, pi_id: Uuid) {
        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(move |db_conn| {
                let name = NonEmptyString::new(random_string()).unwrap();

                lab::Creation::builder()
                    .name(name.clone())
                    .delivery_dir(name)
                    .pi_id(pi_id)
                    .build()
                    .execute(db_conn)
                    .unwrap();
            })
            .await
            .unwrap();
    }

    // fn insert_specimens(&mut self, db_conn: &mut PgConnection) {
    //     for i in 0..N_SPECIMENS {
    //         let measurement = NewSpecimenMeasurement::builder()
    //             .measured_by(self.random_person_id())
    //             .data(specimen::common::MeasurementData::Rin {
    //                 measured_at: self.random_time(),
    //                 instrument_name: Some("mayonnaise".into()),
    //                 value: 5.0,
    //             })
    //             .build();

    //         let random_species = Species::VARIANTS.choose(&mut
    // self.rng).copied().unwrap();         let mut inner_specimen =
    // NewSpecimenCommon::builder()
    // .readable_id(Uuid::now_v7().to_string())
    // .submitted_by(self.random_person_id())
    // .lab_id(self.random_lab_id())
    // .received_at(self.random_time())             .species([random_species])
    //             .tissue("krabby patty")
    //             .measurements([measurement])
    //             .name(format!("specimen{i}"))
    //             .build();

    //         let new_specimen: NewSpecimen = if i % 7 == 0 {
    //             inner_specimen.additional_data = Some(any_value!({"Tissue":
    // "Soul"}));

    //             let s = NewCryopreservedSuspension::builder()
    //                 .inner(inner_specimen)
    //                 .build();

    //             NewSpecimen::CryopreservedSuspension(s)
    //         } else if i % 6 == 0 {
    //             inner_specimen.additional_data = Some(any_value!({"Condition":
    // "Excellent"}));

    //             let s =
    // NewFrozenSuspension::builder().inner(inner_specimen).build();

    //             NewSpecimen::FrozenSuspension(s)
    //         } else if i % 5 == 0 {
    //             inner_specimen.additional_data = Some(any_value!({"Storage
    // Buffer": "Chowder"}));

    //             let s = NewCryopreservedTissue::builder()
    //                 .inner(inner_specimen)
    //                 .build();

    //             NewSpecimen::CryopreservedTissue(s)
    //         } else if i % 4 == 0 {
    //             let s = NewFixedTissue::builder()
    //                 .inner(inner_specimen)
    //                 .fixative(TissueFixative::DithiobisSuccinimidylpropionate)
    //                 .build();

    //             NewSpecimen::FixedTissue(s)
    //         } else if i % 3 == 0 {
    //             let s = NewFrozenTissue::builder().inner(inner_specimen).build();

    //             NewSpecimen::FrozenTissue(s)
    //         } else if i % 2 == 0 {
    //             let s = NewFixedBlock::builder()
    //                 .inner(inner_specimen)
    //                 .fixative(BlockFixative::FormaldehydeDerivative)
    //                 .embedded_in(FixedBlockEmbeddingMatrix::Paraffin)
    //                 .build();

    //             NewSpecimen::FixedBlock(s)
    //         } else {
    //             inner_specimen.additional_data =
    //                 Some(any_value!({"Secret": "the krabby-patty secret
    // formular"}));

    //             let random_embedding_matrix =
    // FrozenBlockEmbeddingMatrix::VARIANTS                 .choose(&mut
    // self.rng)                 .copied()
    //                 .unwrap();

    //             let s = NewFrozenBlock::builder()
    //                 .inner(inner_specimen)
    //                 .embedded_in(random_embedding_matrix)
    //                 .build();

    //             NewSpecimen::FrozenBlock(s)
    //         };

    //         let specimen = new_specimen.execute(db_conn).unwrap();

    //         self.specimens.push(specimen);
    //     }
    // }

    // fn random_specimen_id(&mut self) -> Uuid {
    //     self.specimens.choose_unwrap(&mut self.rng).info.id_
    // }

    // fn random_multiplexing_tag_id(&mut self) -> Uuid {
    //     self.multiplexing_tags.choose_unwrap(&mut self.rng).id
    // }

    // fn suspension_volume(&mut self) ->
    // suspension::common::SuspensionMeasurementFields {
    //     suspension::common::SuspensionMeasurementFields::Volume {
    //         measured_at: self.random_time(),
    //         value: 10.0,
    //         unit: VolumeUnit::Microliter,
    //     }
    // }

    // fn new_suspensions(&mut self, n: usize, for_pool: bool) -> Vec<NewSuspension>
    // {     let mut new_suspensions = Vec::with_capacity(n);
    //     for i in 0..n {
    //         let new_suspension_measurements: Vec<_> = (0..2)
    //             .map(|_| {
    //                 NewSuspensionMeasurement::builder()
    //                     .measured_by(self.random_person_id())
    //                     .data(SuspensionMeasurementData {
    //                         fields: self.suspension_volume(),
    //                         is_post_probe_hybridization: for_pool,
    //                     })
    //                     .build()
    //             })
    //             .collect();

    //         let new_suspension = NewSuspension::builder()
    //             .biological_material(BiologicalMaterial::Cells)
    //             .readable_id(Uuid::now_v7().to_string())
    //             .parent_specimen_id(self.random_specimen_id())
    //             .target_cell_recovery(5_000.0 + i as f32)
    //             .measurements(new_suspension_measurements)
    //             .preparer_ids(self.random_people_ids(2));

    //         let new_suspension = if for_pool {
    //             new_suspension
    //                 .multiplexing_tag_id(self.random_multiplexing_tag_id())
    //                 .build()
    //         } else {
    //             new_suspension.build()
    //         };

    //         new_suspensions.push(new_suspension);
    //     }

    //     // Ensure uniqueness
    //     let mut last_multiplexing_tag_id = Some(Uuid::nil());
    //     for s in &mut new_suspensions {
    //         while s.multiplexing_tag_id == last_multiplexing_tag_id {
    //             s.multiplexing_tag_id = Some(self.random_multiplexing_tag_id());
    //         }
    //         last_multiplexing_tag_id = s.multiplexing_tag_id;
    //     }

    //     new_suspensions
    // }

    // fn insert_suspension_pools(&mut self, db_conn: &mut PgConnection) {
    //     for i in 0..N_SUSPENSION_POOLS {
    //         let new_suspension_pool_measurement =
    // NewSuspensionPoolMeasurement::builder()
    // .measured_by(self.random_person_id())
    // .data(self.suspension_volume())             .build();

    //         let new_suspension_pool = NewSuspensionPool::builder()
    //             .readable_id(Uuid::now_v7().to_string())
    //             .name(format!("pool{i}"))
    //             .pooled_at(self.random_time())
    //             .preparer_ids(self.random_people_ids(2))
    //             .suspensions(self.new_suspensions(N_SUSPENSIONS_PER_POOL, true))
    //             .measurements([new_suspension_pool_measurement])
    //             .build();

    //         self.suspension_pools
    //             .push(new_suspension_pool.execute(db_conn).unwrap());
    //     }
    // }

    // fn random_suspension_pool_id(&mut self) -> Uuid {
    //     self.suspension_pools
    //         .choose_unwrap(&mut self.rng)
    //         .summary
    //         .id
    // }

    // fn flex_assay_id(&self) -> Uuid {
    //     let flex_assays: Vec<_> = self
    //         .tenx_assays
    //         .iter()
    //         .map(|a| a.clone())
    //         .filter(|a| {
    //             a.name == "Flex Gene Expression"
    //                 && a.chemistry_version == "v1 - GEM-X"
    //                 && a.sample_multiplexing ==
    // Some(SampleMultiplexing::FlexBarcode)         })
    //         .collect();

    //     if flex_assays.len() != 1 {
    //         panic!(
    //             "multiple Flex Gene Expression assays found: {:?}",
    //             flex_assays
    //         );
    //     }

    //     flex_assays[0].id
    // }

    // fn insert_pool_multiplexed_chromium_runs(&mut self, db_conn: &mut
    // PgConnection) {     let assay_id = self.flex_assay_id();

    //     for i in 0..N_POOL_MULTIPLEX_CHROMIUM_RUNS {
    //         let chromium_run_common = NewChromiumRunCommon::builder()
    //             .readable_id(format!("PMCR{i}"))
    //             .assay_id(assay_id)
    //             .run_at(self.random_time())
    //             .run_by(self.random_person_id())
    //             .succeeded(true)
    //             .build();

    //         let chip_loading_common = NewChipLoadingCommon::builder()
    //             .suspension_volume_loaded(self.suspension_volume())
    //             .buffer_volume_loaded(self.suspension_volume())
    //             .build();

    //         let gems: Vec<_> = (0..N_GEMS_PER_NONOCM_CHROMIUM_RUN)
    //             .map(|j| {
    //                 let chip_loading = NewPoolMultiplexChipLoading::builder()
    //                     .inner(chip_loading_common.clone())
    //                     .suspension_pool_id(self.random_suspension_pool_id())
    //                     .build();

    //                 NewPoolMultiplexGems::builder()
    //                     .loading([chip_loading])
    //                     .inner(
    //                         NewGemsCommon::builder()
    //                             .readable_id(format!("G{i}-{j}",))
    //                             .build(),
    //                     )
    //                     .build()
    //             })
    //             .collect();

    //         let chromium_run = NewPoolMultiplexChromiumRun::builder()
    //             .inner(chromium_run_common)
    //             .gems(gems)
    //             .build();

    //         self.chromium_runs.push(
    //             NewChromiumRun::PoolMultiplex(chromium_run)
    //                 .execute(db_conn)
    //                 .unwrap(),
    //         );
    //     }
    // }

    // fn insert_cdna(&mut self, db_conn: &mut PgConnection) {
    //     let flex_assay_id = self.flex_assay_id();

    //     // Clone here so we can use `self.random_*`
    //     for chromium_run in self.chromium_runs.clone() {
    //         for gems in &chromium_run.gems {
    //             let new_cdna_measurements = [NewCdnaMeasurement::builder()
    //                 .measured_by(self.random_person_id())
    //                 .data(MeasurementData::Electrophoretic {
    //                     measured_at: self.random_time(),
    //                     instrument_name: "trumpet".into(),
    //                     mean_size_bp: None,
    //                     sizing_range: (50, 1000),
    //                     concentration: Concentration {
    //                         value: 5000.0,
    //                         unit: (MassUnit::Picogram, VolumeUnit::Microliter),
    //                     },
    //                 })
    //                 .build()];

    //             let gems_id = gems.id;

    //             let (cdna_lib_types_and_volumes, lib_volumes_and_index_sets) =
    //                 if chromium_run.info.assay.id == flex_assay_id {
    //                     (
    //                         vec![(LibraryType::GeneExpression, 100.0)],
    //                         vec![(40.0, "TS")],
    //                     )
    //                 } else {
    //                     unreachable!("all Chromium runs are instances of Flex
    // Gene Expression")                 };

    //             let cdna = cdna_lib_types_and_volumes
    //                 .into_iter()
    //                 .map(|(ty, cdna_vol)| {
    //                     NewCdna::builder()
    //                         .library_type(ty)
    //                         .gems_id(gems_id)
    //                         .volume_µl(cdna_vol)
    //                         .readable_id(format!("C{gems_id}"))
    //                         .prepared_at(self.random_time())
    //                         .n_amplification_cycles(7)
    //                         .preparer_ids(self.random_people_ids(2))
    //                         .measurements(new_cdna_measurements.clone())
    //                         .build()
    //                 })
    //                 .collect();

    //             if chromium_run.info.assay.id == flex_assay_id {
    //                 let cdnas = NewCdnaGroup::Multiple { cdna
    // }.execute(db_conn).unwrap();

    //                 self.cdna_groups.push(
    //                     cdnas
    //                         .into_iter()
    //                         .zip(lib_volumes_and_index_sets)
    //                         .map(|(cdna, (lib_vol, index_set))| (cdna, lib_vol,
    // index_set))                         .collect(),
    //                 );
    //             }
    //         }
    //     }
    // }

    // fn insert_libraries(&mut self, db_conn: &mut PgConnection) {
    //     for cdna_group in self.cdna_groups.clone() {
    //         let mut library_group = Vec::with_capacity(cdna_group.len());

    //         for (i, (cdna, lib_vol, index_set)) in
    // cdna_group.into_iter().enumerate() {             let
    // new_library_measurement = [NewLibraryMeasurement::builder()
    // .measured_by(self.random_person_id())
    // .data(MeasurementData::Fluorometric {                     measured_at:
    // self.random_time(),                     instrument_name: "hubble".into(),
    //                     concentration: Concentration {
    //                         value: 30.0,
    //                         unit: (MassUnit::Nanogram, VolumeUnit::Microliter),
    //                     },
    //                 })
    //                 .build()];

    //             let cdna_id = cdna.summary.id;

    //             let new_library = NewLibrary::builder()
    //                 .readable_id(format!("L{cdna_id}"))
    //                 .cdna_id(cdna_id)
    //                 .dual_index_set_name(format!("SI-{index_set}-A{}", i + 1))
    //                 .measurements(new_library_measurement)
    //                 .prepared_at(self.random_time())
    //                 .preparer_ids(self.random_people_ids(2))
    //                 .number_of_sample_index_pcr_cycles(10)
    //                 .target_reads_per_cell(50_000)
    //                 .volume_µl(lib_vol)
    //                 .build();

    //             let library = new_library.clone().execute(db_conn).unwrap();
    //             library_group.push(library);
    //         }

    //         self.libraries.push(library_group);
    //     }
    // }

    // fn insert_sequencing_runs(&mut self, db_conn: &mut PgConnection) {
    //     let time = self.random_time();
    //     let libraries: Vec<_> = self
    //         .libraries
    //         .iter()
    //         .flat_map(|libraries| {
    //             libraries.iter().map(|l| {
    //                 NewSequencingSubmission::builder()
    //                     .library_id(l.info.id_)
    //                     .submitted_at(time)
    //                     .build()
    //             })
    //         })
    //         .collect();

    //     let sequencing_run = NewSequencingRun::builder()
    //         .readable_id(format!("SR{}", Uuid::now_v7()))
    //         .begun_at(self.random_time())
    //         .finished_at(self.random_time())
    //         .libraries(libraries)
    //         .build()
    //         .execute(db_conn)
    //         .unwrap();

    //     self.sequencing_runs.push(sequencing_run);
    // }

    // fn insert_chromium_datasets(&mut self, db_conn: &mut PgConnection) {
    //     for library_group in self.libraries.clone() {
    //         let library_ids: Vec<_> = library_group.iter().map(|l|
    // l.info.id_).collect();

    //         let inner = NewChromiumDatasetCommon::builder()
    //             .name("dataset")
    //             .lab_id(self.random_lab_id())
    //             .data_path("path")
    //             .delivered_at(self.random_time())
    //             .library_ids(library_ids)
    //             .web_summaries([String::new()])
    //             .build();

    //         let library_types: Vec<_> = library_group
    //             .iter()
    //             .map(|l| l.info.cdna.library_type)
    //             .collect();

    //         let dataset = if library_types == [LibraryType::GeneExpression] {
    //             let metrics: Vec<_> = (0..N_SUSPENSIONS_PER_POOL)
    //                 .map(|_| MultiRowCsvMetricsFile {
    //                     filename: "metrics".to_string(),
    //                     raw_contents: include_str!(
    //
    // "models/dataset/chromium/test-data/cellranger_multi.csv"
    // )                     .into(),
    //                     contents: Vec::default(),
    //                 })
    //                 .collect();

    //             NewChromiumDataset::CellrangerMulti(NewCellrangerMultiDataset {
    //                 inner,
    //                 metrics: metrics.into(),
    //             })
    //         } else {
    //             unreachable!("only multiplexed Flex Gene Expression is
    // supported")         };

    //         self.chromium_datasets
    //             .push(dataset.execute(db_conn).unwrap());
    //     }
    // }

    async fn root_db_conn(&self) -> Connection {
        self.root_db_pool.get().await.unwrap()
    }

    async fn all<Q, T>(&self) -> Vec<T>
    where
        Q: std::fmt::Debug + NoLimit + db::Operation<Vec<T>>,
        T: 'static + Send,
    {
        let db_conn = self.root_db_conn().await;

        db_conn
            .interact(|db_conn| Q::no_limit().execute(db_conn).unwrap())
            .await
            .unwrap()
    }

    async fn all_extract<Q, T, F, U>(&self, f: F) -> Vec<U>
    where
        Q: std::fmt::Debug + NoLimit + db::Operation<Vec<T>>,
        T: 'static + Send,
        F: Fn(&T) -> U,
    {
        self.all::<Q, _>().await.iter().map(f).collect()
    }
}

#[derive(Debug, Default)]
pub struct Database {
    pub institutions: Vec<institution::Institution>,
    pub people: Vec<person::Summary>,
    pub labs: Vec<lab::Summary>,
}

impl Database {
    async fn new(test_state: &'static TestState) -> Self {
        test_state.populate_db().await;

        let (institutions, people, labs) = tokio::join!(
            test_state.all::<institution::Query, _>(),
            test_state.all::<person::Query, _>(),
            test_state.all::<lab::Query, _>()
        );

        Self {
            institutions,
            people,
            labs,
        }
    }
}

fn random_string() -> String {
    let mut rng = rand::rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

// These numbers correspond to the first second of the year -4000 and the last second of the year 4000 (https://www.postgresql.org/docs/current/datatype-datetime.html)
const TIME: Range<i64> = -188_395_009_438..64_092_229_199;

fn random_time() -> Timestamp {
    let mut rng = SmallRng::seed_from_u64(0);
    Timestamp::from_second(TIME.choose(&mut rng).unwrap()).unwrap()
}

const N_INSTITUTIONS: usize = 50;
const N_PEOPLE: usize = 200;
const N_LABS: usize = 100;
pub const N_LAB_MEMBERS: usize = 5;

pub const N_SPECIMENS: usize = 1000;

const N_MULTIPLEXING_TAGS: usize = 1600;

// 25% of the specimens will be pooled
pub const N_SUSPENSION_POOLS: usize = N_SPECIMENS / 4;
pub const N_SUSPENSIONS_PER_POOL: usize = 2;

// The remaining specimens will become singular suspensions
pub const N_SUSPENSIONS: usize = N_SPECIMENS - (N_SUSPENSION_POOLS * N_SUSPENSIONS_PER_POOL);

const N_TENX_ASSAYS: usize = 15;

const N_GEMS_PER_NONOCM_CHROMIUM_RUN: usize = 8;
const N_GEMS_PER_OCM_CHROMIUM_RUN: usize = 2;
const N_SUSPENSIONS_PER_OCM_GEMS: usize = 4;

// Every suspension can be used both for singleplex and OCM runs
const N_SINGLEPLEX_CHROMIUM_RUNS: usize = N_SUSPENSIONS / N_GEMS_PER_NONOCM_CHROMIUM_RUN;
const N_OCM_CHROMIUM_RUNS: usize =
    N_SUSPENSIONS / (N_GEMS_PER_OCM_CHROMIUM_RUN * N_SUSPENSIONS_PER_OCM_GEMS);

// Every suspension pool can be used for a pool multiplex chromium run
pub const N_POOL_MULTIPLEX_CHROMIUM_RUNS: usize =
    N_SUSPENSION_POOLS / N_GEMS_PER_NONOCM_CHROMIUM_RUN;

const N_CDNA: usize = (N_SINGLEPLEX_CHROMIUM_RUNS * N_GEMS_PER_NONOCM_CHROMIUM_RUN)
    + (N_OCM_CHROMIUM_RUNS * N_GEMS_PER_OCM_CHROMIUM_RUN)
    + (N_POOL_MULTIPLEX_CHROMIUM_RUNS * N_GEMS_PER_NONOCM_CHROMIUM_RUN);

const N_LIBRARIES: usize = N_CDNA;

const N_SEQUENCING_RUNS: usize = 1;

const N_CHROMIUM_DATASETS: usize = N_LIBRARIES;

trait ChooseUnwrap<T> {
    fn choose_unwrap(&self) -> T;
}

impl<T> ChooseUnwrap<T> for [T]
where
    T: Copy,
{
    fn choose_unwrap(&self) -> T {
        let mut rng = SmallRng::seed_from_u64(0);
        *self.choose(&mut rng).unwrap()
    }
}
