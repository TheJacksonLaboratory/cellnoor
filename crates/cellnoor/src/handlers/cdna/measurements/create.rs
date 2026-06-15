use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::nucleic_acid_measurement::NewNucleicAcidMeasurement;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
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

    let response = insert_cdna_measurement(&tx, cdna_id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn insert_cdna_measurement(
    tx: &db::Transaction<'_>,
    cdna_id: Uuid,
    record: &NewNucleicAcidMeasurement,
) -> Result<(), ErrorInner> {
    db::insert_into_no_returning(tx, "cdna_measurement", &(cdna_id, record)).await?;

    Ok(())
}

impl AsFieldValuePairs<&'static str, 4> for (Uuid, &NewNucleicAcidMeasurement) {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 4> {
        let (
            cdna_id,
            NewNucleicAcidMeasurement {
                measured_by,
                measured_at,
                data,
            },
        ) = self;

        [
            ("cdna_id", cdna_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
