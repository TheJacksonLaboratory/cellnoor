use std::assert_matches;

use cellnoor_types::{
    operator::UuidOperator,
    permission::{Action, Permission, Resource},
    project::{NewProject, ProjectDetailed, ProjectPredicate},
    specimen::{
        SpecimenDetailed, SpecimenPredicate, SpecimenQuery, creation::NewSpecimen,
        measurement::NewSpecimenMeasurement,
    },
};
use jiff::Timestamp;
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    handlers::{
        permissions::grant_permissions,
        projects::{create::insert_test_project, index_detailed::select_projects_detailed},
        security_tests::{
            permissions::{create_test_user_with, may},
            rls::project::create_project_as,
        },
        specimens::{
            create::{insert_specimen, insert_test_specimen_and_project},
            index_detailed::select_specimens_detailed,
            measurements::{
                create::insert_specimen_measurements, remove::delete_specimen_measurements,
            },
            update::update_specimen_by_id,
        },
    },
    state::dev_util::{db_client_as_admin, db_client_as_user},
};

async fn insert_accessible_specimen(tx: &db::Transaction<'_>) -> (NewSpecimen, SpecimenDetailed) {
    // The underlying `insert_test_project` adds one person to the project, so
    // we don't need to do anything extra here
    insert_test_specimen_and_project(tx, |_| ()).await.unwrap()
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

    insert_test_specimen_and_project(tx, |s| s.simple.project_id = project.record.project.id)
        .await
        .unwrap()
}

pub async fn get_user_id_from_specimen(
    tx: &db::Transaction<'_>,
    specimen: &SpecimenDetailed,
) -> Uuid {
    let projects = select_projects_detailed(
        tx,
        &ProjectPredicate::Id(UuidOperator::Eq(specimen.record.project_id)).into(),
    )
    .await
    .unwrap();

    projects[0].record.members[0]
}

async fn test_user_can_only_see_accessible_specimen(
    tx: &db::Transaction<'_>,
    accessible_specimen: SpecimenDetailed,
) {
    let specimens = select_specimens_detailed(tx, &SpecimenQuery::default())
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
        tx,
        &SpecimenPredicate::Id(UuidOperator::Eq(inaccessible_specimen_id)).into(),
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

    // Commit this transaction so the change persists for the next part of the
    // test
    tx.commit().await.unwrap();

    // Log in as the new user
    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    test_user_can_only_see_accessible_specimen(&tx, accessible_specimen).await;
    test_user_cannot_see_inaccessible_specimen(&tx, *inaccessible_specimen.record.id).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn member_can_create_specimen_only_in_accessible_project() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    // The user is a member of the accessible specimen's project only, and may
    // create specimens
    let ((new_specimen, accessible_specimen), (_, inaccessible_project)) = tokio::join!(
        insert_accessible_specimen(&tx),
        insert_inaccessible_project(&tx)
    );
    let user_id = get_user_id_from_specimen(&tx, &accessible_specimen).await;
    grant_permissions(&tx, user_id, &may(Resource::Specimen, &[Action::Create]))
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let mut new = new_specimen;

    let mut client = db_client_as_user(user_id).await;

    // This needs its own transaction because the failure aborts it
    let tx = client.begin().await.unwrap();
    let mut into_inaccessible_project = new.clone();
    into_inaccessible_project.simple.readable_id =
        crate::db::dummy_data::random_name_for("specimen");
    into_inaccessible_project.simple.project_id = inaccessible_project.record.project.id;

    let error = insert_specimen(&tx, into_inaccessible_project)
        .await
        .unwrap_err();
    assert_eq!(
        error,
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"specimen\""
                .to_owned()
        }
    );
    drop(tx);

    let tx = client.begin().await.unwrap();
    new.simple.readable_id = crate::db::dummy_data::random_name_for("specimen");

    insert_specimen(&tx, new).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn creator_can_see_specimens_of_project_without_access() {
    let creator_id = create_test_user_with(false, may(Resource::Project, &[Action::Create])).await;

    let project = create_project_as(creator_id, vec![]).await;

    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (_, specimen) =
        insert_test_specimen_and_project(&tx, |s| s.simple.project_id = project.record.project.id)
            .await
            .unwrap();

    tx.commit().await.unwrap();

    let mut client = db_client_as_user(creator_id).await;
    let tx = client.begin().await.unwrap();

    test_user_can_only_see_accessible_specimen(&tx, specimen).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn only_staff_with_permission_can_update_specimen() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (new_specimen, specimen) = insert_accessible_specimen(&tx).await;
    let member_id = get_user_id_from_specimen(&tx, &specimen).await;

    grant_permissions(&tx, member_id, &may(Resource::Specimen, &[Action::Update]))
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let specimen_id = *specimen.record.id;
    let staff_without_permission = create_test_user_with(true, Vec::new()).await;
    let staff_with_permission =
        create_test_user_with(true, may(Resource::Specimen, &[Action::Update])).await;

    let mut update = new_specimen.simple;
    update.name = crate::db::dummy_data::random_name_for("specimen");

    // The member holds the permission but isn't staff, and the other user is
    // staff without the permission
    for user_id in [member_id, staff_without_permission] {
        let mut client = db_client_as_user(user_id).await;
        let tx = client.begin().await.unwrap();

        assert_eq!(
            update_specimen_by_id(&tx, specimen_id, update.clone())
                .await
                .unwrap_err(),
            DbError::ResourceNotFound
        );
    }

    let mut client = db_client_as_user(staff_with_permission).await;
    let tx = client.begin().await.unwrap();

    update_specimen_by_id(&tx, specimen_id, update)
        .await
        .unwrap();
}

// Unlike other measurements, members may add specimen measurements, but only to
// their own projects' specimens. Removing them is still left to staff
#[tokio::test(flavor = "multi_thread")]
async fn member_can_add_specimen_measurements_only_in_accessible_project() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let ((new_specimen, specimen), (_, inaccessible_specimen)) = tokio::join!(
        insert_accessible_specimen(&tx),
        insert_inaccessible_specimen(&tx)
    );
    let member_id = get_user_id_from_specimen(&tx, &specimen).await;

    grant_permissions(
        &tx,
        member_id,
        &may(Resource::Specimen, &[Action::Create, Action::Delete]),
    )
    .await
    .unwrap();

    tx.commit().await.unwrap();

    let id = *specimen.record.id;
    let inaccessible_id = *inaccessible_specimen.record.id;
    let new_measurement = NewSpecimenMeasurement {
        measured_at: Timestamp::now(),
        ..new_specimen.measurements[0].clone()
    };
    let new_measurements = std::slice::from_ref(&new_measurement);
    let measurement_ids: Vec<_> = specimen.measurements.iter().map(|m| m.id).collect();

    let mut client = db_client_as_user(member_id).await;

    // Each failure needs its own transaction because it aborts it
    let tx = client.begin().await.unwrap();
    assert_matches!(
        insert_specimen_measurements(&tx, inaccessible_id, new_measurements)
            .await
            .unwrap_err(),
        DbError::PermissionDenied { .. }
    );
    drop(tx);

    let tx = client.begin().await.unwrap();
    insert_specimen_measurements(&tx, id, new_measurements)
        .await
        .unwrap();
    assert_eq!(
        delete_specimen_measurements(&tx, id, &measurement_ids)
            .await
            .unwrap_err(),
        DbError::ResourceNotFound
    );
    drop(tx);

    // Staff need no membership, and either `create` and `delete`, or `update`
    for actions in [&[Action::Create, Action::Delete][..], &[Action::Update]] {
        let staff_id = create_test_user_with(true, may(Resource::Specimen, actions)).await;

        let mut client = db_client_as_user(staff_id).await;
        let tx = client.begin().await.unwrap();

        insert_specimen_measurements(&tx, inaccessible_id, new_measurements)
            .await
            .unwrap();
        delete_specimen_measurements(&tx, id, &measurement_ids)
            .await
            .unwrap();
    }
}
