use cellnoor_types::nonempty::NonemptyString;
use strum::VariantArray;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    handlers::{
        chromium_datasets::{insert_test_chromium_dataset, write_test_dataset_files},
        chromium_runs::TestChromiumRunKind,
    },
    state::{AppState, dev_util::ToNonemptyString},
};

pub fn random_name_for(table: &str) -> NonemptyString {
    format!("{table}-{}", &Uuid::new_v4().to_string()[0..8]).to_nonempty_string()
}

pub async fn populate_dummy_data(app_state: AppState) {
    let mut client = app_state.db_client(AuthUser::dev_admin()).await.unwrap();
    let tx = client.begin().await.unwrap();

    let mut datasets = Vec::with_capacity(100);
    for run_kind in TestChromiumRunKind::VARIANTS {
        for _ in 0..25 {
            let (_, ds) = insert_test_chromium_dataset(&tx, *run_kind, |_| ())
                .await
                .unwrap();

            datasets.push(*ds.record.id);
        }
    }

    tx.commit().await.unwrap();

    for ds in datasets {
        write_test_dataset_files(ds).await;
    }
}
