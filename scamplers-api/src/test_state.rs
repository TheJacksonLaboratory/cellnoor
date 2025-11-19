use std::collections::HashSet;

use diesel::{PgConnection, prelude::*};
use jiff::Timestamp;
use non_empty_string::non_empty_string;
use rand::{
    SeedableRng,
    rngs::StdRng,
    seq::{IndexedRandom, IteratorRandom},
};
use rstest::fixture;
use scamplers_models::{institution::InstitutionId, person::PersonId, *};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{config::Config, db::Operation, state::AppState};

trait ChooseUnwrap<T> {
    fn choose_unwrap(&self, rng: &mut StdRng) -> &T;
}
impl<T> ChooseUnwrap<T> for Vec<T> {
    fn choose_unwrap(&self, rng: &mut StdRng) -> &T {
        self.choose(rng).unwrap()
    }
}
trait ChooseUnwrapOwned<T> {
    fn choose_unwrap_owned(self, rng: &mut StdRng) -> T;
}
impl ChooseUnwrapOwned<i64> for std::ops::Range<i64> {
    fn choose_unwrap_owned(self, rng: &mut StdRng) -> i64 {
        self.choose(rng).unwrap()
    }
}

const N_INSTITUTIONS: usize = 10;
const N_PEOPLE: usize = 250;
const N_LABS: usize = 50;
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

pub struct TestState {
    rng: StdRng,
    app_state: AppState,
    institutions: Vec<institution::Institution>,
    people: Vec<person::Person>,
    labs: Vec<lab::Lab>,
    // specimens: Vec<Specimen>,
    // multiplexing_tags: Vec<MultiplexingTag>,
    // suspension_pools: Vec<SuspensionPool>,
    // tenx_assays: Vec<TenxAssay>,
    // chromium_runs: Vec<ChromiumRun>,
    // cdna_groups: Vec<Vec<(Cdna, f32, &'static str)>>,
    // libraries: Vec<Vec<Library>>,
    // sequencing_runs: Vec<SequencingRun>,
    // chromium_datasets: Vec<ChromiumDataset>,
}
impl TestState {
    fn random_time(&mut self) -> Timestamp {
        // These numbers correspond to the first second of the year -4000 and the last second of the year 4000 (https://www.postgresql.org/docs/current/datatype-datetime.html)
        Timestamp::from_second(
            (-188_395_009_438..64_092_229_199).choose_unwrap_owned(&mut self.rng),
        )
        .unwrap()
    }

    fn insert_institutions(&mut self, db_conn: &mut PgConnection) {
        for i in 0..N_INSTITUTIONS {
            let new_institution = institution::Creation::builder()
                .id(Uuid::now_v7())
                .name(non_empty_string!("institution{i}"))
                .build()
                .execute(db_conn)
                .unwrap();

            self.institutions.push(new_institution);
        }

        // We know that initial_data.sample.json contains the Jackson Laboratory so make
        // sure to add that too
        let query = institution::Query::builder()
            .filter(
                institution::Filter::builder()
                    .name("Jackson Laboratory")
                    .build(),
            )
            .build();
        let jax = query.execute(db_conn).unwrap().remove(0);
        let jax = InstitutionId(jax.id()).execute(db_conn).unwrap();
        self.institutions.push(jax);
    }

    fn random_institution_id(&mut self) -> Uuid {
        self.institutions.choose_unwrap(&mut self.rng).id()
    }

    fn insert_people(&mut self, db_conn: &mut PgConnection) {
        for i in 0..N_PEOPLE {
            let new_person = person::Creation::builder()
                .name(non_empty_string!("person{i}"))
                .email(non_empty_string!("person{i}@example.com"))
                .institution_id(self.random_institution_id())
                .roles([])
                .build()
                .execute(db_conn)
                .unwrap();

            self.people.push(new_person);
        }

        // We know that initial_data contains "ahmed said" so make sure we add that too
        let query = person::Query::builder()
            .filter(person::Filter::builder().name("ahmed").build())
            .build();
        let handsome_cool_dude = query.execute(db_conn).unwrap().remove(0);
        let handsome_cool_dude = PersonId(handsome_cool_dude.id()).execute(db_conn).unwrap();
        self.people.push(handsome_cool_dude);
    }

    fn random_person_id(&mut self) -> Uuid {
        self.people.choose_unwrap(&mut self.rng).id()
    }

    fn insert_labs(&mut self, db_conn: &mut PgConnection) {
        for i in 0..N_LABS {
            let pi_id = self.random_person_id();
            let name = non_empty_string!("lab{i}");

            let new_lab = lab::Creation::builder()
                .name(name.clone())
                .pi_id(pi_id)
                .delivery_dir(non_empty_string!("{name}_dir"))
                .build()
                .execute(db_conn)
                .unwrap();

            self.labs.push(new_lab);
        }
    }

    fn random_lab_id(&mut self) -> Uuid {
        self.labs.choose_unwrap(&mut self.rng).id()
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

    async fn populate_db(mut self) -> Self {
        let db_conn = self.app_state.db_conn().await.unwrap();

        db_conn
            .interact(|db_conn| {
                self.insert_institutions(db_conn);
                self.insert_people(db_conn);
                self.insert_labs(db_conn);
                // self.insert_specimens(db_conn);
                // self.insert_suspension_pools(db_conn);
                // self.insert_pool_multiplexed_chromium_runs(db_conn);
                // self.insert_cdna(db_conn);
                // self.insert_libraries(db_conn);
                // self.insert_sequencing_runs(db_conn);
                // self.insert_chromium_datasets(db_conn);

                self
            })
            .await
            .unwrap()
    }

    async fn new() -> Self {
        // This will read configuration values from the environment
        let config = Config::read().unwrap();

        let test_state = Self {
            app_state: AppState::initialize(&config).await.unwrap(),
            rng: StdRng::from_rng(&mut rand::rng()),
            institutions: Vec::with_capacity(N_INSTITUTIONS + 1),
            people: Vec::with_capacity(N_PEOPLE + 1),
            labs: Vec::with_capacity(N_LABS),
            // specimens: Vec::with_capacity(N_SPECIMENS),
            // suspension_pools: Vec::with_capacity(N_SUSPENSION_POOLS),
            // multiplexing_tags: Vec::with_capacity(N_MULTIPLEXING_TAGS),
            // tenx_assays: Vec::with_capacity(N_TENX_ASSAYS),
            // chromium_runs: Vec::with_capacity(
            //     N_SINGLEPLEX_CHROMIUM_RUNS + N_OCM_CHROMIUM_RUNS +
            // N_POOL_MULTIPLEX_CHROMIUM_RUNS, ),
            // cdna_groups: Vec::with_capacity(N_CDNA),
            // libraries: Vec::with_capacity(N_LIBRARIES),
            // sequencing_runs: Vec::with_capacity(N_SEQUENCING_RUNS),
            // chromium_datasets: Vec::with_capacity(N_CHROMIUM_DATASETS),
        };

        let test_state = test_state.populate_db().await;

        test_state
    }
}

static TEST_STATE: OnceCell<TestState> = OnceCell::const_new();

#[fixture]
pub async fn db_conn() -> deadpool_diesel::postgres::Connection {
    let test_state = TEST_STATE.get_or_init(TestState::new).await;

    test_state.app_state.db_conn().await.unwrap()
}

#[rstest::fixture]
pub async fn institutions() -> Vec<institution::Institution> {
    TEST_STATE
        .get_or_init(TestState::new)
        .await
        .institutions
        .clone()
}
