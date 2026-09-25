use crate::{
    auth::AuthUser, handlers::chromium_datasets::insert_test_chromium_dataset, state::AppState,
};

pub async fn populate_dummy_data(app_state: AppState) {
    let mut client = app_state.db_client(AuthUser::dev_admin()).await.unwrap();
    let tx = client.begin().await.unwrap();

    for _ in 0..100 {
        insert_test_chromium_dataset(&tx, |_| ()).await.unwrap();
    }

    tx.commit().await.unwrap();
}
