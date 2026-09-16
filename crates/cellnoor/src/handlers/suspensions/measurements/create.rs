use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{Relation, suspension::measurement::NewSuspensionMeasurement};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
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

    let response =
        insert_suspension_measurements(&tx, suspension_id, std::slice::from_ref(&record))
            .await
            .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn insert_suspension_measurements(
    tx: &db::Transaction<'_>,
    suspension_id: Uuid,
    records: &[NewSuspensionMeasurement],
) -> Result<(), ErrorInner> {
    let rows: Vec<_> = records
        .iter()
        .map(|record| NewSuspensionMeasurementRow {
            suspension_id,
            record,
        })
        .collect();

    tx.insert_many(&rows).await
}

struct NewSuspensionMeasurementRow<'a> {
    suspension_id: Uuid,
    record: &'a NewSuspensionMeasurement,
}

impl Relation for NewSuspensionMeasurementRow<'_> {
    const NAME: &'static str = "suspension_measurement";
}

impl Insert for NewSuspensionMeasurementRow<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            suspension_id,
            record:
                NewSuspensionMeasurement {
                    measured_by,
                    measured_at,
                    data,
                },
        } = self;

        vec![
            ("suspension_id", suspension_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
