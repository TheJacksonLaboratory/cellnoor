use cellnoor_types::{
    operator::UuidOperator,
    permission::{Action, Resource},
    project::{NewProject, ProjectDetailed, ProjectPredicate, ProjectQuery, ProjectSimpleFields},
};
use jiff::Timestamp;
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    handlers::{
        permissions::grant_permissions,
        projects::{
            access::{add_people::insert_project_accesses, remove_people::delete_project_accesses},
            create::{insert_project, insert_test_project},
            index_detailed::select_projects_detailed,
            update::update_project_by_id,
        },
        security_tests::permissions::{create_test_user, create_test_user_with, may},
    },
    state::dev_util::{db_client_as_admin, db_client_as_user},
};

async fn insert_accessible_project(tx: &db::Transaction<'_>) -> (NewProject, ProjectDetailed) {
    insert_test_project(tx, |_| ()).await.unwrap()
}

pub async fn insert_inaccessible_project(
    tx: &db::Transaction<'_>,
) -> (NewProject, ProjectDetailed) {
    insert_test_project(tx, |p| p.members = vec![])
        .await
        .unwrap()
}

pub async fn create_project_as(creator_id: Uuid, members: Vec<Uuid>) -> ProjectDetailed {
    let mut client = db_client_as_user(creator_id).await;
    let tx = client.begin().await.unwrap();

    let project = insert_project(
        &tx,
        &NewProject {
            simple: ProjectSimpleFields {
                name: crate::db::dummy_data::random_name_for("project"),
                started_at: Timestamp::now(),
                ended_at: None,
            },
            members,
        },
    )
    .await
    .unwrap();

    tx.commit().await.unwrap();

    project
}

async fn user_can_only_see_accessible_project(
    tx: &db::Transaction<'_>,
    accessible_project: ProjectDetailed,
) {
    let projects = select_projects_detailed(tx, &ProjectQuery::default())
        .await
        .unwrap();

    assert_eq!(projects, [accessible_project]);
}

async fn user_cannot_see_inaccessible_project(
    tx: &db::Transaction<'_>,
    inaccessible_project_id: Uuid,
) {
    let res = select_projects_detailed(
        tx,
        &ProjectPredicate::Id(UuidOperator::Eq(inaccessible_project_id)).into(),
    )
    .await
    .unwrap();

    assert_eq!(res, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_projects() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    // Insert a project the user can access and one the user cannot
    let ((_, accessible_project), (_, inaccessible_project)) = tokio::join!(
        insert_accessible_project(&tx),
        insert_inaccessible_project(&tx)
    );

    // We have to commit this transaction so the change persists for the next
    // part of the test
    tx.commit().await.unwrap();

    let person_id = accessible_project.record.members[0];

    // Log in as the new user
    let mut client = db_client_as_user(person_id).await;
    let tx = client.begin().await.unwrap();

    user_can_only_see_accessible_project(&tx, accessible_project).await;
    user_cannot_see_inaccessible_project(&tx, inaccessible_project.record.project.id).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn nonstaff_user_can_create_project() {
    let user_id = create_test_user_with(false, may(Resource::Project, &[Action::Create])).await;

    let project = create_project_as(user_id, vec![]).await;

    assert_eq!(project.record.project.created_by, user_id);
}

#[tokio::test(flavor = "multi_thread")]
async fn creator_can_see_project_without_access() {
    let creator_id = create_test_user_with(false, may(Resource::Project, &[Action::Create])).await;

    let project = create_project_as(creator_id, vec![]).await;

    let mut client = db_client_as_user(creator_id).await;
    let tx = client.begin().await.unwrap();

    user_can_only_see_accessible_project(&tx, project).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn only_staff_with_permission_can_update_project() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (new_project, project) = insert_accessible_project(&tx).await;
    let member_id = project.record.members[0];

    grant_permissions(&tx, member_id, &may(Resource::Project, &[Action::Update]))
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let project_id = project.record.project.id;
    let staff_without_permission = create_test_user_with(true, Vec::new()).await;
    let staff_with_permission =
        create_test_user_with(true, may(Resource::Project, &[Action::Update])).await;

    let mut update = new_project.simple;
    update.name = crate::db::dummy_data::random_name_for("project");

    // The member holds the permission but isn't staff, and the other user is
    // staff without the permission
    for user_id in [member_id, staff_without_permission] {
        let mut client = db_client_as_user(user_id).await;
        let tx = client.begin().await.unwrap();

        assert_eq!(
            update_project_by_id(&tx, project_id, &update)
                .await
                .unwrap_err(),
            DbError::ResourceNotFound
        );
    }

    let mut client = db_client_as_user(staff_with_permission).await;
    let tx = client.begin().await.unwrap();

    update_project_by_id(&tx, project_id, &update)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn non_member_cannot_grant_project_access() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (_, project) = insert_inaccessible_project(&tx).await;

    tx.commit().await.unwrap();

    // The outsider may create projects, so only the membership check can
    // refuse them
    let outsider_id = create_test_user_with(false, may(Resource::Project, &[Action::Create])).await;

    let mut client = db_client_as_user(outsider_id).await;
    let tx = client.begin().await.unwrap();

    let error = insert_project_accesses(&tx, project.record.project.id, &[outsider_id])
        .await
        .unwrap_err();

    assert_eq!(
        error,
        DbError::PermissionDenied {
            message: "new row violates row-level security policy for resource \"project_access\""
                .to_owned()
        }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn creator_can_grant_first_project_access() {
    let creator_id = create_test_user_with(false, may(Resource::Project, &[Action::Create])).await;
    let member_id = create_test_user().await;

    let project = create_project_as(creator_id, vec![member_id]).await;

    assert_eq!(project.record.members, [member_id]);
}

#[tokio::test(flavor = "multi_thread")]
async fn member_can_grant_project_access() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (_, project) = insert_accessible_project(&tx).await;

    tx.commit().await.unwrap();

    let member_id = project.record.members[0];
    let new_member_id = create_test_user().await;

    let mut client = db_client_as_user(member_id).await;
    let tx = client.begin().await.unwrap();

    insert_project_accesses(&tx, project.record.project.id, &[new_member_id])
        .await
        .unwrap();
}

// Any member may add people (`member_can_grant_project_access`), but only the
// creator may remove them. Staff who may delete projects can too
#[tokio::test(flavor = "multi_thread")]
async fn only_creator_can_remove_project_members() {
    let creator_id = create_test_user_with(false, may(Resource::Project, &[Action::Create])).await;
    let member_id = create_test_user_with(
        false,
        may(
            Resource::Project,
            &[Action::Create, Action::Update, Action::Delete],
        ),
    )
    .await;
    let other_member_id = create_test_user().await;

    let project = create_project_as(creator_id, vec![member_id, other_member_id]).await;
    let project_id = project.record.project.id;

    // Even with every permission on projects, a member who didn't create the
    // project can't remove anyone
    let mut client = db_client_as_user(member_id).await;
    let tx = client.begin().await.unwrap();

    assert_eq!(
        delete_project_accesses(&tx, project_id, &[other_member_id])
            .await
            .unwrap_err(),
        DbError::ResourceNotFound
    );

    let mut client = db_client_as_user(creator_id).await;
    let tx = client.begin().await.unwrap();

    delete_project_accesses(&tx, project_id, &[other_member_id])
        .await
        .unwrap();

    let staff_id = create_test_user_with(true, may(Resource::Project, &[Action::Delete])).await;
    let mut client = db_client_as_user(staff_id).await;
    let tx = client.begin().await.unwrap();

    delete_project_accesses(&tx, project_id, &[member_id])
        .await
        .unwrap();
}
