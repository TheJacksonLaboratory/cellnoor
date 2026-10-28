use cellnoor_types::{nucleic_acid_measurement::NewNucleicAcidMeasurement, permission::Resource};
use jiff::Timestamp;

use crate::{
    handlers::{
        chromium_runs::create::TestChromiumRunKind,
        libraries::{
            create::{insert_library_preparers, insert_test_library},
            measurements::{
                create::insert_library_measurements, remove::delete_library_measurements,
            },
            preparers::remove::delete_library_preparers,
        },
        security_tests::permissions::{
            assert_only_staff_with_permission_can_change_children, create_test_user,
        },
    },
    state::dev_util::db_client_as_admin,
};

#[tokio::test(flavor = "multi_thread")]
async fn only_staff_with_permission_can_change_library_preparers_and_measurements() {
    let mut client = db_client_as_admin().await;
    let tx = client.begin().await.unwrap();

    let (new_library, library) = insert_test_library(&tx, TestChromiumRunKind::Standard, |_| ())
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let id = *library.record.id;
    let member_id = library.preparers[0];
    let new_preparer_id = create_test_user().await;
    let new_measurement = NewNucleicAcidMeasurement {
        measured_at: Timestamp::now(),
        ..new_library.measurements[0].clone()
    };
    let measurement_ids: Vec<_> = library.measurements.iter().map(|m| m.id).collect();

    assert_only_staff_with_permission_can_change_children(
        Resource::ChromiumExperimentalData,
        member_id,
        async |tx| insert_library_preparers(tx, id, &[new_preparer_id]).await,
        async |tx| delete_library_preparers(tx, id, &library.preparers).await,
    )
    .await;

    assert_only_staff_with_permission_can_change_children(
        Resource::ChromiumExperimentalData,
        member_id,
        async |tx| {
            insert_library_measurements(tx, id, std::slice::from_ref(&new_measurement)).await
        },
        async |tx| delete_library_measurements(tx, id, &measurement_ids).await,
    )
    .await;
}
