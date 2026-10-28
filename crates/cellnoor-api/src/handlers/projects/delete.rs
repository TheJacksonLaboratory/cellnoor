#[cfg(test)]
mod test {
    use cellnoor_types::project::SavedProjectRecord;

    use crate::{
        handlers::projects::create::insert_test_project, state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_project(&tx, |_| ()).await.unwrap();
        tx.delete::<SavedProjectRecord>(inserted.record().id)
            .await
            .unwrap();
    }
}
