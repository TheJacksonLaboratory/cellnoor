use std::fmt::Debug;

use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::{
        institutions::show::select_institution_by_id,
        people::create::test::insert_test_person_and_institution,
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

async fn test_no_error<F, T>(tx: &db::Transaction<'_>, select_fn: F)
where
    T: Debug,
    F: AsyncFn(&db::Transaction, Uuid) -> Result<T, ErrorInner>,
{
    static RANDOM_ID: Uuid = Uuid::max();

    // In this test, we just want to see that we don't get a PermissionDenied
    let err = select_fn(tx, RANDOM_ID).await.unwrap_err();
    pretty_assertions::assert_eq!(err, ErrorInner::ResourceNotFound)
}

#[tokio::test(flavor = "multi_thread")]
async fn user_can_access_every_view() {
    let user_id = create_test_user().await;

    let mut client = db_client_as_user(user_id).await;
    let tx = &client.begin().await.unwrap();

    tokio::join!(test_no_error(tx, select_institution_by_id));
}
