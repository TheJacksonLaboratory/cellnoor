#[cfg(test)]
mod test {
    use cellnoor_types::person::PersonUpdate;

    use crate::{
        handlers::people::create::insert_test_person_and_institution,
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        tx.delete::<PersonUpdate>(person.record.id).await.unwrap();
    }
}
