use cellnoor_types::suspension::{NewSuspension, SuspensionDetailed, SuspensionQuery};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::{
        projects::show::select_project_by_id,
        security_tests::rls::specimen::insert_inaccessible_specimen,
        suspensions::{
            create::test::insert_test_suspension_and_specimen,
            index_detailed::select_suspensions_detailed, show::select_suspension_by_id,
        },
    },
    state::test_util::{db_client_as_admin, db_client_as_user},
};

async fn insert_accessible_suspension(
    tx: &db::Transaction<'_>,
) -> (NewSuspension, SuspensionDetailed) {
    // The underlying `insert_test_project` adds one person to the project, so we
    // don't need to do anything extra here
    insert_test_suspension_and_specimen(&tx, |_| ())
        .await
        .unwrap()
}

async fn insert_inaccessible_suspension(
    tx: &db::Transaction<'_>,
) -> (NewSuspension, SuspensionDetailed) {
    let (_, specimen) = insert_inaccessible_specimen(tx).await;

    insert_test_suspension_and_specimen(&tx, |s| s.record.specimen_id = *specimen.record.id)
        .await
        .unwrap()
}

async fn get_user_id_from_suspension(
    tx: &db::Transaction<'_>,
    suspension: &SuspensionDetailed,
) -> Uuid {
    let project = select_project_by_id(&tx, suspension.specimen.record.project_id)
        .await
        .unwrap();

    project.record.people[0]
}

async fn test_user_can_only_see_accessible_suspension(
    tx: &db::Transaction<'_>,
    accessible_suspension: SuspensionDetailed,
) {
    // Note that by querying the detailed view, we are querying
    // `suspension_detailed`, and that allows us to test whether a view built on top
    // of a security_invoker = true view still adheres to RLS
    let suspensions = select_suspensions_detailed(&tx, &mut SuspensionQuery::default())
        .await
        .unwrap();

    assert_eq!(suspensions, vec![accessible_suspension]);
}

async fn test_user_cannot_see_inaccessible_suspension(
    tx: &db::Transaction<'_>,
    inaccessible_suspension_id: Uuid,
) {
    // Check that the inaccessible project causes a `ResourceNotFound`
    let error = select_suspension_by_id(&tx, inaccessible_suspension_id)
        .await
        .unwrap_err();

    assert_eq!(error, ErrorInner::ResourceNotFound);
}

#[tokio::test(flavor = "multi_thread")]
async fn row_level_security_for_suspensions() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    // Insert a specimen the user can access and one the user cannot
    let ((_, accessible_suspension), (_, inaccessible_suspension)) = tokio::join!(
        insert_accessible_suspension(&tx),
        insert_inaccessible_suspension(&tx)
    );
    let user_id = get_user_id_from_suspension(&tx, &accessible_suspension).await;

    // Commit this transaction so the change persists for the next part of the test
    tx.commit().await.unwrap();

    // Log in as the new user
    let mut client = db_client_as_user(user_id).await;
    let tx = client.begin().await.unwrap();

    test_user_can_only_see_accessible_suspension(&tx, accessible_suspension).await;
    test_user_cannot_see_inaccessible_suspension(&tx, *inaccessible_suspension.record.id).await;
}
