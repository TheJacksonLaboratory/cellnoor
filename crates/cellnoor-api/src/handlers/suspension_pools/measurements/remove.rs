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

pub async fn remove_suspension_pool_measurements(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: pool_id }): Path<IdParam>,
    crate::extract::JsonExtractor(measurements): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            delete_suspension_pool_measurements(tx, pool_id, &measurements).await
        })
        .await
}

pub(in super::super::super) async fn delete_suspension_pool_measurements(
    tx: &db::Transaction<'_>,
    pool_id: Uuid,
    measurements: &[Uuid],
) -> Result<(), DbError> {
    static DELETE_SUSPENSION_POOL_MEASUREMENTS: &str =
        "delete from suspension_pool_measurement where pool_id = $1 and id = any($2)";

    let n = tx
        .execute(&Sql::new(DELETE_SUSPENSION_POOL_MEASUREMENTS, vec![&pool_id, &measurements]))
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
            suspension_pools::{
                create::insert_test_suspension_pool_and_suspensions, measurements::remove::delete_suspension_pool_measurements,
                show::select_suspension_pool_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_measurements() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, suspension_pool) = insert_test_suspension_pool_and_suspensions(&tx, |_| ()).await.unwrap();
        let (_, other_suspension_pool) = insert_test_suspension_pool_and_suspensions(&tx, |_| ()).await.unwrap();

        let ids: Vec<_> = suspension_pool.measurements.iter().map(|m| m.id).collect();
        let other_ids: Vec<_> = other_suspension_pool.measurements.iter().map(|m| m.id).collect();

        // The other suspension_pool's measurements don't belong to `suspension_pool`, so nothing is deleted
        assert_eq!(
            delete_suspension_pool_measurements(&tx, *suspension_pool.record.id, &other_ids)
                .await
                .unwrap_err(),
            DbError::ResourceNotFound
        );

        delete_suspension_pool_measurements(&tx, *suspension_pool.record.id, &ids)
            .await
            .unwrap();

        let selected = select_suspension_pool_by_id(&tx, *suspension_pool.record.id).await.unwrap();
        let other_selected = select_suspension_pool_by_id(&tx, *other_suspension_pool.record.id)
            .await
            .unwrap();

        assert!(selected.measurements.is_empty());
        assert_eq!(other_selected.measurements, other_suspension_pool.measurements);
    }
}
