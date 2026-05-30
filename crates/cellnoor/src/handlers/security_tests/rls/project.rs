use cellnoor_types::project::{NewProject, ProjectDetailed, ProjectQuery};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::projects::{
        create::test::insert_test_project, index_detailed::select_projects_detailed,
        show::select_project_by_id,
    },
    state::test_util::{db_client_as_admin, db_client_as_user},
};

async fn insert_accessible_project(tx: &db::Transaction<'_>) -> (NewProject, ProjectDetailed) {
    insert_test_project(&tx, |_| ()).await.unwrap()
}

pub async fn insert_inaccessible_project(
    tx: &db::Transaction<'_>,
) -> (NewProject, ProjectDetailed) {
    insert_test_project(&tx, |p| p.people = vec![])
        .await
        .unwrap()
}

async fn test_user_can_only_see_accessible_project(
    tx: &db::Transaction<'_>,
    accessible_project: ProjectDetailed,
) {
    let projects = select_projects_detailed(&tx, &mut ProjectQuery::default())
        .await
        .unwrap();

    assert_eq!(projects, vec![accessible_project]);
}

async fn test_user_cannot_see_inaccessible_project(
    tx: &db::Transaction<'_>,
    inaccessible_project_id: Uuid,
) {
    // Check that the inaccessible project causes a `ResourceNotFound`
    let error = select_project_by_id(&tx, inaccessible_project_id)
        .await
        .unwrap_err();

    assert_eq!(error, ErrorInner::ResourceNotFound);
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

    // We have to commit this transaction so the change persists for the next part
    // of the test
    tx.commit().await.unwrap();

    let person_id = accessible_project.record.people[0];

    // Log in as the new user
    let mut client = db_client_as_user(person_id).await;
    let tx = client.begin().await.unwrap();

    test_user_can_only_see_accessible_project(&tx, accessible_project).await;
    test_user_cannot_see_inaccessible_project(&tx, *inaccessible_project.record.project.id).await;
}
