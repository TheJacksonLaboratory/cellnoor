use std::assert_matches;

use cellnoor_types::{
    operator::UuidOperator,
    permission::{Action, Permission, Resource},
    project::{NewProject, ProjectDetailed, ProjectPredicate},
    specimen::{SpecimenDetailed, SpecimenPredicate, SpecimenQuery, creation::NewSpecimen},
};
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

    insert_test_specimen_and_project(tx, |s| s.project_id = project.record.project.id)
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
async fn user_cannot_write_specimen_into_inaccessible_project() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    // The user is a member of the accessible specimen's project only, and may
    // create and update specimens
    let ((new_specimen, accessible_specimen), (_, inaccessible_project)) = tokio::join!(
        insert_accessible_specimen(&tx),
        insert_inaccessible_project(&tx)
    );
    let user_id = get_user_id_from_specimen(&tx, &accessible_specimen).await;
    grant_permissions(
        &tx,
        user_id,
        &[Action::Create, Action::Update].map(|action| Permission {
            resource: Resource::Specimen,
            action,
        }),
    )
    .await
    .unwrap();

    tx.commit().await.unwrap();

    let inaccessible_project_id = inaccessible_project.record.project.id;

    // Log in as the new user
    let mut client = db_client_as_user(user_id).await;

    let tx = client.begin().await.unwrap();
    let mut new = new_specimen.clone();
    new.readable_id = crate::db::dummy_data::random_name_for("specimen");
    new.project_id = inaccessible_project_id;

    let error = insert_specimen(&tx, new).await.unwrap_err();
    assert_eq!(
        error,
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"specimen\""
                .to_owned()
        }
    );
    drop(tx);

    let tx = client.begin().await.unwrap();
    let mut update = new_specimen;
    update.project_id = inaccessible_project_id;
    update.measurements = vec![];

    let error = update_specimen_by_id(&tx, *accessible_specimen.record.id, update)
        .await
        .unwrap_err();
    assert_eq!(
        error,
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"specimen\""
                .to_owned()
        }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn creator_can_see_specimens_of_project_without_access() {
    let creator_id = create_test_user_with(false, may(Resource::Project, &[Action::Create])).await;

    let project = create_project_as(creator_id, vec![]).await;

    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (_, specimen) =
        insert_test_specimen_and_project(&tx, |s| s.project_id = project.record.project.id)
            .await
            .unwrap();

    tx.commit().await.unwrap();

    let mut client = db_client_as_user(creator_id).await;
    let tx = client.begin().await.unwrap();

    test_user_can_only_see_accessible_specimen(&tx, specimen).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn specimen_update_requires_membership_and_permission() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (mut update, specimen) = insert_accessible_specimen(&tx).await;
    let member_id = get_user_id_from_specimen(&tx, &specimen).await;

    tx.commit().await.unwrap();

    let specimen_id = *specimen.record.id;
    let outsider_id =
        create_test_user_with(false, may(Resource::Specimen, &[Action::Update])).await;

    update.name = crate::db::dummy_data::random_name_for("specimen");
    update.measurements = vec![];

    // An outsider can't update the specimen, even with the permission to
    let mut client = db_client_as_user(outsider_id).await;
    let tx = client.begin().await.unwrap();

    let error = update_specimen_by_id(&tx, specimen_id, update.clone())
        .await
        .unwrap_err();

    assert_matches!(error, DbError::ResourceNotFound);

    // A member can't update the specimen without the permission
    let mut client = db_client_as_user(member_id).await;
    let tx = client.begin().await.unwrap();

    assert_eq!(
        update_specimen_by_id(&tx, specimen_id, update)
            .await
            .unwrap_err(),
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"specimen\""
                .to_owned()
        }
    );
}
