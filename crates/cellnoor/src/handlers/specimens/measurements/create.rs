use axum::{Json, extract::State};
use cellnoor_types::specimen::{
    NewSpecimen, Specimen, SpecimenCommonFields, SpecimenVariableFields,
    measurement::{NewSpecimenMeasurement, SpecimenMeasurementData},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToFieldListPlaceholdersParams},
    },
    error::{Error, ErrorInner},
    handlers::specimens::show::select_specimen_by_id,
    state::AppState,
};

pub fn create_specimen_measurement() {}

fn validate_specimen_measurement(
    NewSpecimenMeasurement {
        data: postgres_types::Json(data),
        ..
    }: &NewSpecimenMeasurement,
) -> Result<(), ErrorInner> {
    let (field, value, exclusive_max) = match data {
        SpecimenMeasurementData::Rin { value, .. } => ("RIN", value, 10.0),
        SpecimenMeasurementData::Dv200 { value, .. } => ("DV200", value, 1.0),
    };

    if *value > exclusive_max {
        return Err(ErrorInner::DataConstraint {
            resource: Some("specimen_measurement".to_owned()),
            field: Some(field.to_owned()),
            message: format!("invalid value for {field}"),
            detail: None,
        });
    }

    Ok(())
}

async fn insert_specimen_measurement(
    tx: &db::Transaction<'_>,
    specimen_id: Uuid,
    NewSpecimenMeasurement {
        measured_by,
        measured_at,
        data,
    }: &NewSpecimenMeasurement,
) -> Result<(), Error> {
    let fields: FieldValuePairs<_> = [
        ("specimen_id", &specimen_id),
        ("measured_by", measured_by),
        ("measured_at", measured_at),
        ("data", data),
    ];

    let (field_list, placeholders, params) = fields.to_field_list_placeholders_params();

    tx.execute(
        &format!(
            "insert into specimen_measurement {field_list} values {placeholders} on conflict do \
             update "
        ),
        &params,
    )
    .await?;

    Ok(())
}
