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
    handlers::path::IdParam,
    state::AppState,
};

pub async fn create_library_measurement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: library_id }): Path<IdParam>,
    Json(record): Json<NewNucleicAcidMeasurement>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_library_measurement(&tx, library_id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_library_measurement(
    tx: &db::Transaction<'_>,
    library_id: Uuid,
    record: &NewNucleicAcidMeasurement,
) -> Result<(), ErrorInner> {
    let row = NewLibraryMeasurement { library_id, record };

    db::insert_into_no_returning(tx, "library_measurement", &row).await?;

    Ok(())
}

struct NewLibraryMeasurement<'a> {
    library_id: Uuid,
    record: &'a NewNucleicAcidMeasurement,
}

impl AsFieldValuePairs<&'static str, 4> for NewLibraryMeasurement<'_> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 4> {
        let Self {
            library_id,
            record:
                NewNucleicAcidMeasurement {
                    measured_by,
                    measured_at,
                    data,
                },
        } = self;

        [
            ("library_id", library_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
