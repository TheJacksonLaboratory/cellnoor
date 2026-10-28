#[cfg(test)]
mod test {
    use cellnoor_types::institution::SavedInstitutionRecord;

    use crate::{
        handlers::institutions::create::insert_test_institution,
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, institution) = insert_test_institution(&tx, |_| ()).await.unwrap();
        tx.delete::<SavedInstitutionRecord>(*institution.record.id)
            .await
            .unwrap();
    }
}
