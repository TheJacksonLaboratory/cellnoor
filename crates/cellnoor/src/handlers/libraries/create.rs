use axum::{Json, extract::State};
use cellnoor_types::library::{LibraryDetailed, LibraryField, NewLibrary, NewLibraryRecord};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::libraries::{
        measurements::create::insert_library_measurement, show::select_library_by_id,
    },
    state::AppState,
};

pub async fn create_library(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewLibrary>,
) -> Result<Json<LibraryDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_library(&tx, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_library(
    tx: &db::Transaction<'_>,
    NewLibrary {
        record,
        measurements,
        preparers,
    }: NewLibrary,
) -> Result<LibraryDetailed, ErrorInner> {
    let id = db::insert_into(tx, "library", &record).await?;

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_library_measurement(tx, id, m)),
    );

    tokio::try_join!(
        insert_library_preparers(tx, id, preparers.as_ref()),
        measurement_insertions
    )?;

    select_library_by_id(tx, id).await
}

pub(super) async fn insert_library_preparers(
    tx: &db::Transaction<'_>,
    library_id: Uuid,
    preparer_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    let preparers: Vec<_> = preparer_ids
        .iter()
        .map(|&prepared_by| NewLibraryPreparer {
            library_id,
            prepared_by,
        })
        .collect();

    futures::future::try_join_all(
        preparers
            .iter()
            .map(|p| db::insert_into_no_returning(tx, "library_preparer", p)),
    )
    .await?;

    Ok(())
}

struct NewLibraryPreparer {
    library_id: Uuid,
    prepared_by: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewLibraryPreparer {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            library_id,
            prepared_by,
        } = self;

        [("library_id", library_id), ("prepared_by", prepared_by)]
    }
}

impl AsFieldValuePairs<LibraryField, 8> for NewLibraryRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, LibraryField, 8> {
        use LibraryField::*;

        let Self {
            id: _,
            readable_id,
            cdna_id,
            single_index_set_name,
            dual_index_set_name,
            number_of_sample_index_pcr_cycles,
            target_reads_per_cell,
            prepared_at,
            additional_data,
        } = self;

        [
            (ReadableId, readable_id),
            (CdnaId, cdna_id),
            (SingleIndexSetName, single_index_set_name),
            (DualIndexSetName, dual_index_set_name),
            (
                NumberOfSampleIndexPcrCycles,
                number_of_sample_index_pcr_cycles,
            ),
            (TargetReadsPerCell, target_reads_per_cell),
            (PreparedAt, prepared_at),
            (AdditionalData, additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        id::NoId,
        library::{LibraryDetailed, NewLibrary, NewLibraryRecord},
        nucleic_acid_measurement::{
            Concentration, NewNucleicAcidMeasurement, NucleicAcidMeasurementData,
        },
        units::{Microliter, Nanogram},
    };
    use jiff::Timestamp;
    use positive::PositiveI32;
    use postgres_types::Json;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            cdna::create::test::insert_test_cdna_and_chromium_run, index_sets::DUAL_INDEX_SET_NAME,
            libraries::create::insert_library,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_library<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewLibrary, LibraryDetailed), ErrorInner>
    where
        F: FnMut(&mut NewLibrary),
    {
        let (_, cdna) = insert_test_cdna_and_chromium_run(tx, |_| ()).await?;

        let person_id = cdna.preparers[0];

        let mut new = NewLibrary {
            record: NewLibraryRecord {
                id: NoId {},
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                cdna_id: *cdna.record.id,
                single_index_set_name: None,
                dual_index_set_name: Some(DUAL_INDEX_SET_NAME.to_owned()),
                number_of_sample_index_pcr_cycles: PositiveI32::new(8).unwrap(),
                target_reads_per_cell: Some(PositiveI32::new(50_000).unwrap()),
                prepared_at: Timestamp::now(),
                additional_data: None,
            },
            measurements: vec![NewNucleicAcidMeasurement {
                measured_by: person_id,
                measured_at: Timestamp::now(),
                data: Json(NucleicAcidMeasurementData::Fluorometric {
                    instrument_name: "Qubit".to_nonempty_string(),
                    concentration: Concentration {
                        value: PositiveI32::new(20).unwrap(),
                        numerator_unit: Nanogram::Nanogram,
                        denominator_unit: Microliter::Microliter,
                    },
                }),
            }],
            preparers: nonempty::NonemptyVec::new(vec![person_id]).unwrap(),
        };

        modify(&mut new);

        let inserted = insert_library(tx, new.clone()).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_library(&tx, |_| ()).await.unwrap();
    }
}
