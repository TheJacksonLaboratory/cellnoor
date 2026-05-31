use cellnoor_types::{
    cdna::{CdnaDetailed, CdnaPredicate, CdnaPredicateInner, CdnaQuery, NewCdna},
    chromium_run::creation::{NewChromiumRun, standard::NewStandardChipLoading},
    operator::UuidOperator,
};
use nonempty::NonemptyBoundedVec;
use uuid::Uuid;

use crate::{
    db,
    handlers::{
        cdna::{
            create::test::insert_test_cdna_and_chromium_run, index_detailed::select_cdna_detailed,
        },
        chromium_runs::create::test::insert_test_standard_chromium_run,
        security_tests::rls::specimen::insert_inaccessible_specimen,
        suspensions::create::test::insert_test_suspension_and_specimen,
    },
    state::test_util::{db_client_as_admin, db_client_as_user},
};

async fn insert_accessible_cdna(tx: &db::Transaction<'_>) -> (NewCdna, CdnaDetailed) {
    // The underlying `insert_test_project` adds one person to the project, so we
    // don't need to do anything extra here
    insert_test_cdna_and_chromium_run(&tx, |_| ())
        .await
        .unwrap()
}

fn make_standard_chromium_run_inaccessible(run: &mut NewChromiumRun, suspension_id: Uuid) {
    let NewChromiumRun::Standard {
        common: _,
        gem_wells,
    } = run
    else {
        panic!("expected standard Chromium run");
    };

    let NewStandardChipLoading::Suspension {
        suspension_id: loaded_suspension,
        common: _,
    } = &mut gem_wells[0].loading
    else {
        panic!("expected suspension to be loaded into GEM well");
    };

    *loaded_suspension = suspension_id;
    *gem_wells = NonemptyBoundedVec::new(vec![gem_wells[0].clone()]).unwrap();
}

async fn insert_inaccessible_cdna(tx: &db::Transaction<'_>) -> (NewCdna, CdnaDetailed) {
    let (_, specimen) = insert_inaccessible_specimen(tx).await;

    let (_, suspension) =
        insert_test_suspension_and_specimen(&tx, |s| s.record.specimen_id = *specimen.record.id)
            .await
            .unwrap();

    let (_, chromium_run) = insert_test_standard_chromium_run(tx, |run| {
        make_standard_chromium_run_inaccessible(run, *suspension.record.id)
    })
    .await
    .unwrap();

    let gem_well_id = *chromium_run.gem_wells[0].record.id;

    insert_test_cdna_and_chromium_run(tx, |c| c.record.gem_well_id = Some(gem_well_id))
        .await
        .unwrap()
}

async fn get_user_id_from_cdna(cdna: &CdnaDetailed) -> Uuid {
    cdna.preparers[0]
}

async fn test_user_can_only_see_accessible_cdna(
    tx: &db::Transaction<'_>,
    accessible_cdna: CdnaDetailed,
) {
    let cdnas = select_cdna_detailed(&tx, &mut CdnaQuery::default())
        .await
        .unwrap();

    assert_eq!(cdnas, [accessible_cdna]);
}

async fn test_user_cannot_see_inaccessible_cdna(
    tx: &db::Transaction<'_>,
    inaccessible_cdna_id: Uuid,
) {
    let pred: CdnaPredicate = CdnaPredicateInner::Id(UuidOperator::Eq(inaccessible_cdna_id)).into();

    let res = select_cdna_detailed(&tx, &mut pred.into()).await.unwrap();

    assert_eq!(res, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_cdna() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    // Insert a cDNA the user can access and one the user cannot
    let ((_, accessible_cdna), (_, inaccessible_cdna)) =
        tokio::join!(insert_accessible_cdna(&tx), insert_inaccessible_cdna(&tx));
    let user_id = get_user_id_from_cdna(&accessible_cdna).await;

    // Commit this transaction so the change persists for the next part of the test
    tx.commit().await.unwrap();

    // Log in as the new user
    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    test_user_can_only_see_accessible_cdna(&tx, accessible_cdna).await;
    test_user_cannot_see_inaccessible_cdna(&tx, *inaccessible_cdna.record.id).await;
}
