use cellnoor_types::query::ComplexQuery;
use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::{
        institutions::index::select_institutions,
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

async fn test_no_error<F, Pred, OrderField, Return>(tx: &db::Transaction<'_>, select_fn: F)
where
    F: AsyncFn(&db::Transaction, &mut ComplexQuery<Pred, OrderField>) -> Result<Return, ErrorInner>,
    OrderField: Default,
{
    // For this test, we just want to see no error
    select_fn(tx, &mut ComplexQuery::default()).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn user_can_access_every_view() {
    let user_id = create_test_user().await;

    let mut client = db_client_as_user(user_id).await;
    let tx = &client.begin().await.unwrap();

    tokio::join!(test_no_error(tx, select_institutions));
}
