#[cfg(test)]
mod test {
    use cellnoor_types::bot_account::BotAccountSimpleFields;

    use crate::{
        handlers::bot_accounts::create::test::insert_test_bot_account,
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_bot_account(&tx, |_| ()).await.unwrap();
        tx.delete::<BotAccountSimpleFields>(inserted.id)
            .await
            .unwrap();
    }
}
