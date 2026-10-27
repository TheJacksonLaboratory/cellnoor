use cellnoor_types::{permission::Resource, suspension_pool::measurement::NewSuspensionPoolMeasurement};
use jiff::Timestamp;

use crate::{
    handlers::{
        security_tests::permissions::{
            assert_only_staff_with_permission_can_change_children, create_test_user,
        },
        suspension_pools::{
            create::{insert_suspension_pool_preparers, insert_test_suspension_pool_and_suspensions},
            measurements::{
                create::insert_suspension_pool_measurements, remove::delete_suspension_pool_measurements,
            },
            preparers::remove::delete_suspension_pool_preparers,
        },
    },
    state::dev_util::db_client_as_admin,
};

#[tokio::test(flavor = "multi_thread")]
async fn only_staff_with_permission_can_change_suspension_pool_preparers_and_measurements() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (new_suspension_pool, suspension_pool) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let id = *suspension_pool.record.id;
    let member_id = suspension_pool.preparers[0];
    let new_preparer_id = create_test_user().await;
    let new_measurement = NewSuspensionPoolMeasurement {
        measured_at: Timestamp::now(),
        ..new_suspension_pool.measurements[0].clone()
    };
    let measurement_ids: Vec<_> = suspension_pool.measurements.iter().map(|m| m.id).collect();

    assert_only_staff_with_permission_can_change_children(
        Resource::ChromiumExperimentalData,
        member_id,
        async |tx| insert_suspension_pool_preparers(tx, id, &[new_preparer_id]).await,
        async |tx| delete_suspension_pool_preparers(tx, id, &suspension_pool.preparers).await,
    )
    .await;

    assert_only_staff_with_permission_can_change_children(
        Resource::ChromiumExperimentalData,
        member_id,
        async |tx| {
            insert_suspension_pool_measurements(tx, id, std::slice::from_ref(&new_measurement)).await
        },
        async |tx| delete_suspension_pool_measurements(tx, id, &measurement_ids).await,
    )
    .await;
}
