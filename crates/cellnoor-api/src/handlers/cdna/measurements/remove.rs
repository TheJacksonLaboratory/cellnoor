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

pub async fn remove_cdna_measurements(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: cdna_id }): Path<IdParam>,
    crate::extract::JsonExtractor(measurements): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            delete_cdna_measurements(tx, cdna_id, &measurements).await
        })
        .await
}

pub(in super::super::super) async fn delete_cdna_measurements(
    tx: &db::Transaction<'_>,
    cdna_id: Uuid,
    measurements: &[Uuid],
) -> Result<(), DbError> {
    static DELETE_CDNA_MEASUREMENTS: &str =
        "delete from cdna_measurement where cdna_id = $1 and id = any($2)";

    let n = tx
        .execute(&Sql::new(
            DELETE_CDNA_MEASUREMENTS,
            vec![&cdna_id, &measurements],
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
        handlers::{
            cdna::{
                create::insert_test_cdna_and_chromium_run,
                measurements::remove::delete_cdna_measurements, show::select_cdna_by_id,
            },
            chromium_runs::create::TestChromiumRunKind,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_measurements() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, cdna) =
            insert_test_cdna_and_chromium_run(&tx, TestChromiumRunKind::Standard, |_| ())
                .await
                .unwrap();
        let (_, other_cdna) =
            insert_test_cdna_and_chromium_run(&tx, TestChromiumRunKind::Standard, |_| ())
                .await
                .unwrap();

        let ids: Vec<_> = cdna.measurements.iter().map(|m| m.id).collect();
        let other_ids: Vec<_> = other_cdna.measurements.iter().map(|m| m.id).collect();

        // The other cdna's measurements don't belong to `cdna`, so nothing is
        // deleted
        assert_eq!(
            delete_cdna_measurements(&tx, *cdna.record.id, &other_ids)
                .await
                .unwrap_err(),
            DbError::ResourceNotFound
        );

        delete_cdna_measurements(&tx, *cdna.record.id, &ids)
            .await
            .unwrap();

        let selected = select_cdna_by_id(&tx, *cdna.record.id).await.unwrap();
        let other_selected = select_cdna_by_id(&tx, *other_cdna.record.id).await.unwrap();

        assert!(selected.measurements.is_empty());
        assert_eq!(other_selected.measurements, other_cdna.measurements);
    }
}
