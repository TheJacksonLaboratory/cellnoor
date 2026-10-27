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

pub async fn remove_library_measurements(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: library_id }): Path<IdParam>,
    crate::extract::JsonExtractor(measurements): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            delete_library_measurements(tx, library_id, &measurements).await
        })
        .await
}

pub(in super::super::super) async fn delete_library_measurements(
    tx: &db::Transaction<'_>,
    library_id: Uuid,
    measurements: &[Uuid],
) -> Result<(), DbError> {
    static DELETE_LIBRARY_MEASUREMENTS: &str =
        "delete from library_measurement where library_id = $1 and id = any($2)";

    let n = tx
        .execute(&Sql::new(
            DELETE_LIBRARY_MEASUREMENTS,
            vec![&library_id, &measurements],
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
            chromium_runs::create::TestChromiumRunKind,
            libraries::{
                create::insert_test_library, measurements::remove::delete_library_measurements,
                show::select_library_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_measurements() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, library) = insert_test_library(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();
        let (_, other_library) = insert_test_library(&tx, TestChromiumRunKind::Standard, |_| ())
            .await
            .unwrap();

        let ids: Vec<_> = library.measurements.iter().map(|m| m.id).collect();
        let other_ids: Vec<_> = other_library.measurements.iter().map(|m| m.id).collect();

        // The other library's measurements don't belong to `library`, so
        // nothing is deleted
        assert_eq!(
            delete_library_measurements(&tx, *library.record.id, &other_ids)
                .await
                .unwrap_err(),
            DbError::ResourceNotFound
        );

        delete_library_measurements(&tx, *library.record.id, &ids)
            .await
            .unwrap();

        let selected = select_library_by_id(&tx, *library.record.id).await.unwrap();
        let other_selected = select_library_by_id(&tx, *other_library.record.id)
            .await
            .unwrap();

        assert!(selected.measurements.is_empty());
        assert_eq!(other_selected.measurements, other_library.measurements);
    }
}
