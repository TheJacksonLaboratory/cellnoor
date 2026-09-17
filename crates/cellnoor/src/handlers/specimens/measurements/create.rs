use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{Relation, specimen::measurement::NewSpecimenMeasurement};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn create_specimen_measurement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<NewSpecimenMeasurement>,
) -> Result<Json<()>, Error> {
    state
        .in_transaction(user, async |tx| {
            insert_specimen_measurements(tx, id, std::slice::from_ref(&record)).await
        })
        .await
}

pub(in super::super) async fn insert_specimen_measurements(
    tx: &db::Transaction<'_>,
    specimen_id: Uuid,
    records: &[NewSpecimenMeasurement],
) -> Result<(), ErrorInner> {
    let rows: Vec<_> = records
        .iter()
        .map(|record| NewSpecimenMeasurementRow {
            specimen_id,
            record,
        })
        .collect();

    tx.insert_many(&rows).await
}

struct NewSpecimenMeasurementRow<'a> {
    specimen_id: Uuid,
    record: &'a NewSpecimenMeasurement,
}

impl Relation for NewSpecimenMeasurementRow<'_> {
    const NAME: &'static str = "specimen_measurement";
}

impl Insert for NewSpecimenMeasurementRow<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            specimen_id,
            record:
                NewSpecimenMeasurement {
                    measured_by,
                    measured_at,
                    data,
                },
        } = self;

        vec![
            ("specimen_id", specimen_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
