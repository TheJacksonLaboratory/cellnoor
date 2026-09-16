use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{Relation, nucleic_acid_measurement::NewNucleicAcidMeasurement};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn create_cdna_measurement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: cdna_id }): Path<IdParam>,
    Json(record): Json<NewNucleicAcidMeasurement>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_cdna_measurements(&tx, cdna_id, std::slice::from_ref(&record))
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn insert_cdna_measurements(
    tx: &db::Transaction<'_>,
    cdna_id: Uuid,
    records: &[NewNucleicAcidMeasurement],
) -> Result<(), ErrorInner> {
    let rows: Vec<_> = records
        .iter()
        .map(|record| NewCdnaMeasurementRow { cdna_id, record })
        .collect();

    tx.insert_many(&rows).await
}

struct NewCdnaMeasurementRow<'a> {
    cdna_id: Uuid,
    record: &'a NewNucleicAcidMeasurement,
}

impl Relation for NewCdnaMeasurementRow<'_> {
    const NAME: &'static str = "cdna_measurement";
}

impl Insert for NewCdnaMeasurementRow<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            cdna_id,
            record:
                NewNucleicAcidMeasurement {
                    measured_by,
                    measured_at,
                    data,
                },
        } = self;

        vec![
            ("cdna_id", cdna_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
