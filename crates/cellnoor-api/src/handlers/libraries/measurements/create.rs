use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{Relation, nucleic_acid_measurement::NewNucleicAcidMeasurement};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::IdParam,
    state::AppState,
};

pub async fn create_library_measurement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: library_id }): Path<IdParam>,
    Json(record): Json<NewNucleicAcidMeasurement>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            insert_library_measurements(tx, library_id, std::slice::from_ref(&record)).await
        })
        .await
}

pub(in super::super) async fn insert_library_measurements(
    tx: &db::Transaction<'_>,
    library_id: Uuid,
    records: &[NewNucleicAcidMeasurement],
) -> Result<(), DbError> {
    let rows: Vec<_> = records
        .iter()
        .map(|record| NewLibraryMeasurement { library_id, record })
        .collect();

    tx.insert_many(&rows).await
}

struct NewLibraryMeasurement<'a> {
    library_id: Uuid,
    record: &'a NewNucleicAcidMeasurement,
}

impl Relation for NewLibraryMeasurement<'_> {
    const NAME: &'static str = "library_measurement";
}

impl Insert for NewLibraryMeasurement<'_> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            library_id,
            record:
                NewNucleicAcidMeasurement {
                    measured_by,
                    measured_at,
                    data,
                },
        } = self;

        vec![
            ("library_id", library_id),
            ("measured_by", measured_by),
            ("measured_at", measured_at),
            ("data", data),
        ]
    }
}
