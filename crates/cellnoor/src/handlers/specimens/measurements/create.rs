use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::measurement::{NewSpecimenMeasurement, SpecimenMeasurementData};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToFieldListPlaceholdersParams},
    },
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
    NewSpecimenMeasurement {
        measured_by,
        measured_at,
        data,
    }: &NewSpecimenMeasurement,
) -> Result<(), ErrorInner> {
    validate_specimen_measurement_data(&data.0)?;

    let fields: FieldValuePairs<_> = [
        ("specimen_id", &specimen_id),
        ("measured_by", measured_by),
        ("measured_at", measured_at),
        ("data", data),
    ];

    let (field_list, placeholders, params) = fields.to_field_list_and_placeholders_and_params();

    tx.execute(
        &format!(
            "insert into specimen_measurement {field_list} values {placeholders} on conflict do \
             nothing"
        ),
        &params,
    )
    .await?;

    Ok(())
}

fn validate_specimen_measurement_data(data: &SpecimenMeasurementData) -> Result<(), ErrorInner> {
    let (field, value, inclusive_max) = match data {
        SpecimenMeasurementData::Rin { value, .. } => ("RIN", value, 10.0),
        SpecimenMeasurementData::Dv200 { value, .. } => ("DV200", value, 1.0),
    };

    if *value > inclusive_max {
        return Err(ErrorInner::DataConstraint {
            resource: Some("specimen_measurement".to_owned()),
            field: Some(field.to_owned()),
            message: format!("invalid value for {field}"),
            detail: None,
        });
    }

    Ok(())
}
