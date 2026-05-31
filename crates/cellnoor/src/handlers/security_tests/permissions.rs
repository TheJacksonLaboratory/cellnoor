use std::fmt::Debug;

use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::{
        cdna::show::select_cdna_by_id,
        chromium_datasets::show::select_chromium_dataset_by_id,
        chromium_runs::show::select_chromium_run_by_id,
        institutions::show::select_institution_by_id,
        libraries::show::select_library_by_id,
        people::{create::test::insert_test_person_and_institution, show::select_person_by_id},
        projects::show::select_project_by_id,
        specimens::show::select_specimen_by_id,
        suspension_pools::show::select_suspension_pool_by_id,
        suspensions::show::select_suspension_by_id,
    },
    state::test_util::{db_client_as_admin, db_client_as_user},
};

async fn create_test_user() -> Uuid {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (_, person) = insert_test_person_and_institution(&tx, |_| ())
        .await
        .unwrap();

    tx.commit().await.unwrap();

    *person.record.id
}

async fn assert_no_resource_found<F, T>(tx: &db::Transaction<'_>, select_fn: F)
where
    T: Debug,
    F: AsyncFn(&db::Transaction, Uuid) -> Result<T, ErrorInner>,
{
    // In this test, we just want to see that we don't get a PermissionDenied
    let err = select_fn(tx, Uuid::max()).await.unwrap_err();
    pretty_assertions::assert_eq!(err, ErrorInner::ResourceNotFound,)
}

#[tokio::test(flavor = "multi_thread")]
async fn user_can_access_every_view() {
    let user_id = create_test_user().await;

    let mut client = db_client_as_user(user_id).await;
    let tx = &client.begin().await.unwrap();

    tokio::join!(
        assert_no_resource_found(tx, select_cdna_by_id),
        assert_no_resource_found(tx, async |tx, id| select_chromium_dataset_by_id(tx, "", id)
            .await),
        assert_no_resource_found(tx, select_chromium_run_by_id),
        assert_no_resource_found(tx, select_institution_by_id),
        assert_no_resource_found(tx, select_library_by_id),
        assert_no_resource_found(tx, select_person_by_id),
        assert_no_resource_found(tx, select_project_by_id),
        assert_no_resource_found(tx, select_specimen_by_id),
        assert_no_resource_found(tx, select_suspension_by_id),
        assert_no_resource_found(tx, select_suspension_pool_by_id),
    );
}
