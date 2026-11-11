#[cfg(test)]
mod test {
    use cellnoor_types::project::SavedProjectRecord;

    use crate::{
        db::DbError,
        handlers::{
            projects::create::insert_test_project,
            specimens::create::insert_test_specimen_and_project,
        },
        state::dev_util::db_client_as_admin,
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

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_with_specimen_fails() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, specimen) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let error = tx
            .delete::<SavedProjectRecord>(specimen.project.record.id)
            .await
            .unwrap_err();

        std::assert_matches!(error, DbError::InvalidReference { .. });
    }
}
