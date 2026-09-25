#[cfg(test)]
mod test {
    use cellnoor_types::service::ServiceSimpleFields;

    use crate::{
        handlers::services::create::test::insert_test_service, state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_service(&tx, |_| ()).await.unwrap();
        tx.delete::<ServiceSimpleFields>(inserted.id).await.unwrap();
    }
}
