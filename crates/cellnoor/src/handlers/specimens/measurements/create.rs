use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::measurement::NewSpecimenMeasurement;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::path::IdParam,
    state::AppState,
};

pub async fn create_specimen_measurement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<NewSpecimenMeasurement>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_specimen_measurement(&tx, id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_specimen_measurement(
    tx: &db::Transaction<'_>,
    specimen_id: Uuid,
    record: &NewSpecimenMeasurement,
) -> Result<(), ErrorInner> {
    db::insert_into_no_returning(tx, "specimen_measurement", &(specimen_id, record)).await?;

    Ok(())
}

impl AsFieldValuePairs<&'static str, 4> for (Uuid, &NewSpecimenMeasurement) {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 4> {
        let (
            specimen_id,
            NewSpecimenMeasurement {
                measured_by,
                measured_at,
                data,
            },
        ) = self;

        [
            ("specimen_id", specimen_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
