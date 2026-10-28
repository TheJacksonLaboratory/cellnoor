#[cfg(test)]
mod test {
    use cellnoor_types::api_key::ApiKeyUpdate;

    use crate::{
        handlers::api_keys::create::test::insert_test_api_key, state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, api_key) = insert_test_api_key(&tx, |_| ()).await.unwrap();

        tx.delete::<ApiKeyUpdate>(api_key.record.id).await.unwrap();
    }
}
