use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::IdParam,
    state::AppState,
};

pub async fn remove_specimen_measurements(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: specimen_id }): Path<IdParam>,
    crate::extract::JsonExtractor(measurements): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            delete_specimen_measurements(tx, specimen_id, &measurements).await
        })
        .await
}

pub(in super::super::super) async fn delete_specimen_measurements(
    tx: &db::Transaction<'_>,
    specimen_id: Uuid,
    measurements: &[Uuid],
) -> Result<(), DbError> {
    static DELETE_SPECIMEN_MEASUREMENTS: &str =
        "delete from specimen_measurement where specimen_id = $1 and id = any($2)";

    let n = tx
        .execute(&Sql::new(
            DELETE_SPECIMEN_MEASUREMENTS,
            vec![&specimen_id, &measurements],
        ))
        .await?;

    if n == 0 {
        return Err(DbError::ResourceNotFound);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        db::DbError,
        handlers::specimens::{
            create::insert_test_specimen_and_project,
            measurements::remove::delete_specimen_measurements, show::select_specimen_by_id,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_measurements() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, specimen) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let (_, other_specimen) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();

        let ids: Vec<_> = specimen.measurements.iter().map(|m| m.id).collect();
        let other_ids: Vec<_> = other_specimen.measurements.iter().map(|m| m.id).collect();

        // The other specimen's measurements don't belong to `specimen`, so
        // nothing is deleted
        assert_eq!(
            delete_specimen_measurements(&tx, *specimen.record.id, &other_ids)
                .await
                .unwrap_err(),
            DbError::ResourceNotFound
        );

        delete_specimen_measurements(&tx, *specimen.record.id, &ids)
            .await
            .unwrap();

        let selected = select_specimen_by_id(&tx, *specimen.record.id)
            .await
            .unwrap();
        let other_selected = select_specimen_by_id(&tx, *other_specimen.record.id)
            .await
            .unwrap();

        assert!(selected.measurements.is_empty());
        assert_eq!(other_selected.measurements, other_specimen.measurements);
    }
}
