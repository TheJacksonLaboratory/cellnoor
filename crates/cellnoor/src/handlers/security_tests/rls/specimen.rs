use cellnoor_types::{
    operator::UuidOperator,
    project::{NewProject, ProjectDetailed, ProjectPredicate},
    specimen::{SpecimenDetailed, SpecimenPredicate, SpecimenQuery, creation::NewSpecimen},
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db,
    handlers::{
        projects::{create::test::insert_test_project, index_detailed::select_projects_detailed},
        specimens::{
            create::test::insert_test_specimen_and_project,
            index_detailed::select_specimens_detailed,
        },
    },
    state::test_util::{db_client_as_admin, db_client_as_user},
};

async fn insert_accessible_specimen(tx: &db::Transaction<'_>) -> (NewSpecimen, SpecimenDetailed) {
    // The underlying `insert_test_project` adds one person to the project, so we
    // don't need to do anything extra here
    insert_test_specimen_and_project(&tx, |_| ()).await.unwrap()
}

async fn insert_inaccessible_project(tx: &db::Transaction<'_>) -> (NewProject, ProjectDetailed) {
    insert_test_project(tx, |p| p.members = vec![])
        .await
        .unwrap()
}

pub async fn insert_inaccessible_specimen(
    tx: &db::Transaction<'_>,
) -> (NewSpecimen, SpecimenDetailed) {
    let (_, project) = insert_inaccessible_project(tx).await;

    insert_test_specimen_and_project(&tx, |s| s.project_id = project.record.project.id)
        .await
        .unwrap()
}

pub async fn get_user_id_from_specimen(
    tx: &db::Transaction<'_>,
    specimen: &SpecimenDetailed,
) -> Uuid {
    let projects = select_projects_detailed(
        tx,
        &mut ProjectPredicate::Id(UuidOperator::Eq(specimen.record.project_id)).into(),
    )
    .await
    .unwrap();

    projects[0].record.members[0]
}

async fn test_user_can_only_see_accessible_specimen(
    tx: &db::Transaction<'_>,
    accessible_specimen: SpecimenDetailed,
) {
    let specimens = select_specimens_detailed(&tx, &mut SpecimenQuery::default())
        .await
        .unwrap();

    assert_eq!(specimens, [accessible_specimen]);
}

async fn test_user_cannot_see_inaccessible_specimen(
    tx: &db::Transaction<'_>,
    inaccessible_specimen_id: Uuid,
) {
    // Check that the inaccessible project causes a `ResourceNotFound`
    let res = select_specimens_detailed(
        &tx,
        &mut SpecimenPredicate::Id(UuidOperator::Eq(inaccessible_specimen_id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(res, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_specimens() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    // Insert a specimen the user can access and one the user cannot
    let ((_, accessible_specimen), (_, inaccessible_specimen)) = tokio::join!(
        insert_accessible_specimen(&tx),
        insert_inaccessible_specimen(&tx)
    );
    let user_id = get_user_id_from_specimen(&tx, &accessible_specimen).await;

    // Commit this transaction so the change persists for the next part of the test
    tx.commit().await.unwrap();

    // Log in as the new user
    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    test_user_can_only_see_accessible_specimen(&tx, accessible_specimen).await;
    test_user_cannot_see_inaccessible_specimen(&tx, *inaccessible_specimen.record.id).await;
}
