use std::fmt::Debug;

use cellnoor_types::query::ComplexQuery;
use uuid::Uuid;

use crate::{
    db,
    error::ErrorInner,
    handlers::{
        cdna::index_detailed::select_cdna_detailed,
        chromium_datasets::index_detailed::select_chromium_datasets_detailed,
        chromium_runs::index_detailed::select_chromium_runs_detailed,
        institutions::index::select_institutions,
        libraries::index_detailed::select_libraries_detailed,
        people::{create::test::insert_test_person_and_institution, index::select_people},
        projects::index_detailed::select_projects_detailed,
        services::index::select_services,
        specimens::index_detailed::select_specimens_detailed,
        suspension_pools::index_detailed::select_suspension_pools_detailed,
        suspensions::index_detailed::select_suspensions_detailed,
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

    person.record.id
}

async fn assert_is_ok<F, Pred, OrderField, Ret>(tx: &db::Transaction<'_>, select_fn: F)
where
    OrderField: Default,
    F: AsyncFn(&db::Transaction, &mut ComplexQuery<Pred, OrderField>) -> Result<Ret, ErrorInner>,
    Ret: Debug,
{
    std::assert_matches!(select_fn(tx, &mut ComplexQuery::default()).await, Ok(_));
}

#[tokio::test(flavor = "multi_thread")]
async fn user_can_access_every_view() {
    let user_id = create_test_user().await;

    let mut client = db_client_as_user(user_id).await;
    let tx = &client.begin().await.unwrap();

    tokio::join!(
        assert_is_ok(tx, select_cdna_detailed),
        assert_is_ok(tx, async |tx, q| select_chromium_datasets_detailed(
            tx, "", q
        )
        .await),
        assert_is_ok(tx, select_chromium_runs_detailed),
        assert_is_ok(tx, select_institutions),
        assert_is_ok(tx, select_libraries_detailed),
        assert_is_ok(tx, select_people),
        assert_is_ok(tx, select_projects_detailed),
        assert_is_ok(tx, select_services),
        assert_is_ok(tx, select_specimens_detailed),
        assert_is_ok(tx, select_suspensions_detailed),
        assert_is_ok(tx, select_suspension_pools_detailed),
    );
}
