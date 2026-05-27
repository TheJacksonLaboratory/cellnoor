use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension_pool::measurement::NewSuspensionPoolMeasurement;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn create_suspension_pool_measurement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: pool_id }): Path<IdParam>,
    Json(record): Json<NewSuspensionPoolMeasurement>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_suspension_pool_measurement(&tx, pool_id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_suspension_pool_measurement(
    tx: &db::Transaction<'_>,
    pool_id: Uuid,
    record: &NewSuspensionPoolMeasurement,
) -> Result<(), ErrorInner> {
    db::insert_into_no_returning(tx, "suspension_pool_measurement", &(pool_id, record)).await?;

    Ok(())
}

impl AsFieldValuePairs<&'static str, 4> for (Uuid, &NewSuspensionPoolMeasurement) {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 4> {
        let (
            pool_id,
            NewSuspensionPoolMeasurement {
                measured_by,
                measured_at,
                data,
            },
        ) = self;

        [
            ("pool_id", pool_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
