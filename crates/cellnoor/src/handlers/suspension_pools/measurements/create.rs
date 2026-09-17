use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{Relation, suspension_pool::measurement::NewSuspensionPoolMeasurement};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
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
    state
        .in_transaction(user, async |tx| {
            insert_suspension_pool_measurements(tx, pool_id, std::slice::from_ref(&record)).await
        })
        .await
}

pub(in super::super) async fn insert_suspension_pool_measurements(
    tx: &db::Transaction<'_>,
    pool_id: Uuid,
    records: &[NewSuspensionPoolMeasurement],
) -> Result<(), ErrorInner> {
    let rows: Vec<_> = records
        .iter()
        .map(|record| NewSuspensionPoolMeasurementRow { pool_id, record })
        .collect();

    tx.insert_many(&rows).await
}

struct NewSuspensionPoolMeasurementRow<'a> {
    pool_id: Uuid,
    record: &'a NewSuspensionPoolMeasurement,
}

impl Relation for NewSuspensionPoolMeasurementRow<'_> {
    const NAME: &'static str = "suspension_pool_measurement";
}

impl Insert for NewSuspensionPoolMeasurementRow<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            pool_id,
            record:
                NewSuspensionPoolMeasurement {
                    measured_by,
                    measured_at,
                    data,
                },
        } = self;

        vec![
            ("pool_id", pool_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
