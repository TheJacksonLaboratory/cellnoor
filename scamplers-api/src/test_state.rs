#![allow(dead_code)]
use std::ops::Range;

use deadpool_diesel::postgres::{Connection, Pool};
use jiff::Timestamp;
use non_empty::NonEmptyString;
use rand::{
    Rng, SeedableRng,
    distr::Alphanumeric,
    rngs::SmallRng,
    seq::{IndexedRandom, IteratorRandom},
};
use rstest::fixture;
use scamplers_models::{
    generic_query,
    institution::{Institution, InstitutionCreation, InstitutionQuery},
    lab::{LabCreation, LabQuery, LabSummary},
    person::{PersonCreation, PersonQuery, PersonSummary},
    specimen::{
        BlockFixative, CryopreservedSuspensionCreation, CryopreservedTissueCreation,
        FixedBlockCreation, FixedBlockEmbeddingMatrix, FixedTissueCreation, FrozenBlockCreation,
        FrozenBlockEmbeddingMatrix, FrozenSuspensionCreation, FrozenTissueCreation, Species,
        SpecimenCommonFields, SpecimenCreation, SpecimenQuery, SpecimenSummary, TissueFixative,
    },
};
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
        self.insert_specimens().await;
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
                InstitutionCreation::builder()
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
            .all_extract::<InstitutionQuery, _, _, _>(Institution::id)
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

                PersonCreation::builder()
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
            .all_extract::<PersonQuery, _, _, _>(PersonSummary::id)
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

                LabCreation::builder()
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

    async fn insert_specimens(&'static self) {
        let people_ids = self
            .all_extract::<PersonQuery, _, _, _>(PersonSummary::id)
            .await;
        let lab_ids = self.all_extract::<LabQuery, _, _, _>(LabSummary::id).await;

        let join_set: JoinSet<_> = (0..N_SPECIMENS)
            .map(|i| {
                self.insert_random_specimen(i, people_ids.choose_unwrap(), lab_ids.choose_unwrap())
            })
            .collect();

        join_set.join_all().await;
    }

    async fn insert_random_specimen(&self, i: usize, submitted_by: Uuid, lab_id: Uuid) {
        let db_conn = self.root_db_conn().await;

        let inner = SpecimenCommonFields::builder()
            .readable_id(NonEmptyString::new(random_string()).unwrap())
            .name(NonEmptyString::new(random_string()).unwrap())
            .submitted_by(submitted_by)
            .lab_id(lab_id)
            .received_at(random_time())
            .species(Species::VARIANTS.choose_unwrap())
            .tissue(NonEmptyString::new(random_string()).unwrap())
            .additional_data(serde_json::json!({"krabby_patty_formular": "secret"}))
            .build();

        let new_specimen = if i % 7 == 0 {
            let s = CryopreservedSuspensionCreation::builder()
                .inner(inner)
                .build();

            SpecimenCreation::CryopreservedSuspension(s)
        } else if i % 6 == 0 {
            let s = FrozenSuspensionCreation::builder().inner(inner).build();

            SpecimenCreation::FrozenSuspension(s)
        } else if i % 5 == 0 {
            let s = CryopreservedTissueCreation::builder().inner(inner).build();

            SpecimenCreation::CryopreservedTissue(s)
        } else if i % 4 == 0 {
            let s = FixedTissueCreation::builder()
                .inner(inner)
                .fixative(TissueFixative::VARIANTS.choose_unwrap())
                .build();

            SpecimenCreation::FixedTissue(s)
        } else if i % 3 == 0 {
            let s = FrozenTissueCreation::builder().inner(inner).build();

            SpecimenCreation::FrozenTissue(s)
        } else if i % 2 == 0 {
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

#[derive(Debug, Default)]
pub struct Database {
    pub institutions: Vec<Institution>,
    pub people: Vec<PersonSummary>,
    pub labs: Vec<LabSummary>,
    pub specimens: Vec<SpecimenSummary>,
}

impl Database {
    async fn new(test_state: &'static TestState) -> Self {
        test_state.populate_db().await;

        let (institutions, people, labs, specimens) = tokio::join!(
            test_state.all::<InstitutionQuery, _>(),
            test_state.all::<PersonQuery, _>(),
            test_state.all::<LabQuery, _>(),
            test_state.all::<SpecimenQuery, _>()
        );

        Self {
            institutions,
            people,
            labs,
            specimens,
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
    let mut rng = rand::rng();
    Timestamp::from_second(TIME.choose(&mut rng).unwrap()).unwrap()
}

const N_INSTITUTIONS: usize = 50;
const N_PEOPLE: usize = 200;
const N_LABS: usize = 100;
const N_LAB_MEMBERS: usize = 5;

const N_SPECIMENS: usize = 1000;

const N_MULTIPLEXING_TAGS: usize = 1600;

// 25% of the specimens will be pooled
const N_SUSPENSION_POOLS: usize = N_SPECIMENS / 4;
const N_SUSPENSIONS_PER_POOL: usize = 2;

// The remaining specimens will become singular suspensions
const N_SUSPENSIONS: usize = N_SPECIMENS - (N_SUSPENSION_POOLS * N_SUSPENSIONS_PER_POOL);

const N_TENX_ASSAYS: usize = 15;

const N_GEMS_PER_NONOCM_CHROMIUM_RUN: usize = 8;
const N_GEMS_PER_OCM_CHROMIUM_RUN: usize = 2;
const N_SUSPENSIONS_PER_OCM_GEMS: usize = 4;

// Every suspension can be used both for singleplex and OCM runs
const N_SINGLEPLEX_CHROMIUM_RUNS: usize = N_SUSPENSIONS / N_GEMS_PER_NONOCM_CHROMIUM_RUN;
const N_OCM_CHROMIUM_RUNS: usize =
    N_SUSPENSIONS / (N_GEMS_PER_OCM_CHROMIUM_RUN * N_SUSPENSIONS_PER_OCM_GEMS);

// Every suspension pool can be used for a pool multiplex chromium run
const N_POOL_MULTIPLEX_CHROMIUM_RUNS: usize = N_SUSPENSION_POOLS / N_GEMS_PER_NONOCM_CHROMIUM_RUN;

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
