use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension::measurement::NewSuspensionMeasurement;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn create_suspension_measurement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: suspension_id }): Path<IdParam>,
    Json(record): Json<NewSuspensionMeasurement>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_suspension_measurement(&tx, suspension_id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn insert_suspension_measurement(
    tx: &db::Transaction<'_>,
    suspension_id: Uuid,
    record: &NewSuspensionMeasurement,
) -> Result<(), ErrorInner> {
    db::insert_into_no_returning(tx, "suspension_measurement", &(suspension_id, record)).await?;

    Ok(())
}

impl AsFieldValuePairs<&'static str, 4> for (Uuid, &NewSuspensionMeasurement) {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 4> {
        let (
            suspension_id,
            NewSuspensionMeasurement {
                measured_by,
                measured_at,
                data,
            },
        ) = self;

        [
            ("suspension_id", suspension_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
