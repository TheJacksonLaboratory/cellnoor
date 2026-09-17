#[cfg(test)]
mod test {
    use cellnoor_types::chromium_run::SavedChromiumRunRecord;

    use crate::{
        handlers::chromium_runs::create::test::insert_test_standard_chromium_run,
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, run) = insert_test_standard_chromium_run(&tx, |_| ())
            .await
            .unwrap();

        tx.delete::<SavedChromiumRunRecord>(*run.record.id)
            .await
            .unwrap();
    }
}
