use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    IdParam,
    suspension::measurement::{NewSuspensionMeasurement, SuspensionMeasurementData},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToFieldListPlaceholdersParams},
    },
    error::{Error, ErrorInner},
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
        .map(Json);

    tx.commit().await?;

    response
}

pub async fn insert_suspension_measurement(
    tx: &db::Transaction<'_>,
    suspension_id: Uuid,
    NewSuspensionMeasurement {
        measured_by,
        measured_at,
        data,
    }: &NewSuspensionMeasurement,
) -> Result<(), Error> {
    validate_suspension_measurement_data(&data.0)?;

    let fields: FieldValuePairs<_> = [
        ("suspension_id", &suspension_id),
        ("measured_by", measured_by),
        ("measured_at", measured_at),
        ("data", data),
    ];

    let (field_list, placeholders, params) = fields.to_field_list_placeholders_params();

    tx.execute(
        &format!(
            "insert into suspension_measurement {field_list} values {placeholders} on conflict do \
             nothing"
        ),
        &params,
    )
    .await?;

    Ok(())
}

fn validate_suspension_measurement_data(
    data: &SuspensionMeasurementData,
) -> Result<(), ErrorInner> {
    let (field, value, inclusive_max) = match data {
        SuspensionMeasurementData::Viability { inner, .. } => ("Viability", &inner.value, 1.0),
        _ => return Ok(()),
    };

    if *value > inclusive_max {
        return Err(ErrorInner::DataConstraint {
            resource: Some("suspension_measurement".to_owned()),
            field: Some(field.to_owned()),
            message: format!("invalid value for {field}"),
            detail: None,
        });
    }

    Ok(())
}
