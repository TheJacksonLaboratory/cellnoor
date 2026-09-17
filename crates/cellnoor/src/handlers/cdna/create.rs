use axum::{Json, extract::State};
use cellnoor_types::{
    Relation,
    cdna::{CdnaDetailed, CdnaField, CdnaSimpleFields, NewCdnaRecord, creation::NewCdna},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::cdna::{
        measurements::create::insert_cdna_measurements, show::select_cdna_by_id,
        split_new_cdna_for_insertion::split_new_cdna_for_insertion,
    },
    state::AppState,
};

pub async fn create_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewCdna>,
) -> Result<Json<CdnaDetailed>, Error> {
    state
        .in_transaction(user, async |tx| insert_cdna(tx, record).await)
        .await
}

async fn insert_cdna(tx: &db::Transaction<'_>, new: NewCdna) -> Result<CdnaDetailed, ErrorInner> {
    let (record, measurements, preparers) = split_new_cdna_for_insertion(new);

    let id = tx.insert_returning_id(&record).await?;

    let measurement_insertions = insert_cdna_measurements(tx, id, &measurements);

    tokio::try_join!(
        insert_cdna_preparers(tx, id, preparers.as_ref()),
        measurement_insertions
    )?;

    select_cdna_by_id(tx, id).await
}

pub(super) async fn insert_cdna_preparers(
    tx: &db::Transaction<'_>,
    cdna_id: Uuid,
    preparer_ids: &[Uuid],
) -> Result<(), ErrorInner> {
    let preparers: Vec<_> = preparer_ids
        .iter()
        .map(|&prepared_by| NewCdnaPreparer {
            cdna_id,
            prepared_by,
        })
        .collect();

    tx.insert_many(&preparers).await
}

struct NewCdnaPreparer {
    cdna_id: Uuid,
    prepared_by: Uuid,
}

impl Relation for NewCdnaPreparer {
    const NAME: &'static str = "cdna_preparer";
}

impl Insert for NewCdnaPreparer {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            cdna_id,
            prepared_by,
        } = self;

        vec![("cdna_id", cdna_id), ("prepared_by", prepared_by)]
    }
}

impl Insert for CdnaSimpleFields {
    type Field = CdnaField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use CdnaField::*;

        let Self {
            readable_id,
            prepared_at,
            additional_data,
        } = self;

        vec![
            (ReadableId, readable_id),
            (PreparedAt, prepared_at),
            (AdditionalData, additional_data),
        ]
    }
}

impl Insert for NewCdnaRecord {
    type Field = CdnaField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use CdnaField::*;

        let Self {
            id: _,
            readable_id,
            library_type,
            prepared_at,
            gem_well_id,
            // Populated by a database trigger
            gem_well_run_at: _,
            n_amplification_cycles,
            additional_data,
        } = self;

        vec![
            (ReadableId, readable_id),
            (LibraryType, library_type),
            (PreparedAt, prepared_at),
            (GemWellId, gem_well_id),
            (NAmplificationCycles, n_amplification_cycles),
            (AdditionalData, additional_data),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        cdna::{
            CdnaDetailed, CdnaSimpleFields,
            creation::{CdnaVariableFields, NewCdna},
        },
        nucleic_acid_measurement::{
            Concentration, NewNucleicAcidMeasurement, NucleicAcidMeasurementData,
            NucleicAcidMeasurementMethod,
        },
        positive::PositiveI32,
        units::{Microliter, Nanogram},
    };
    use jiff::Timestamp;
    use postgres_types::Json;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            cdna::create::insert_cdna,
            chromium_runs::create::test::insert_test_standard_chromium_run,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_cdna_and_chromium_run<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewCdna, CdnaDetailed), ErrorInner>
    where
        F: FnMut(&mut NewCdna),
    {
        let (_, run) = insert_test_standard_chromium_run(tx, |_| ()).await?;

        let gem_well_id = *run.gem_wells[0].record.id;
        let person_id = run.record.run_by;
        let prepared_at = run.record.run_at;

        let mut new = NewCdna {
            simple: CdnaSimpleFields {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                prepared_at,
                additional_data: None,
            },
            gem_well_id,
            measurements: vec![NewNucleicAcidMeasurement {
                measured_by: person_id,
                measured_at: Timestamp::now(),
                data: Json(NucleicAcidMeasurementData {
                    instrument_name: "Qubit".to_nonempty_string(),
                    method: NucleicAcidMeasurementMethod::Fluorometric {
                        concentration: Concentration {
                            value: PositiveI32::new(50).unwrap(),
                            numerator_unit: Nanogram::Nanogram,
                            denominator_unit: Microliter::Microliter,
                        },
                    },
                }),
            }],
            preparers: cellnoor_types::nonempty::NonemptyVec::new(vec![person_id]).unwrap(),
            variable_fields: CdnaVariableFields::GeneExpression {
                n_amplification_cycles: PositiveI32::new(10).unwrap(),
            },
        };

        modify(&mut new);

        let inserted = insert_cdna(tx, new.clone()).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_cdna_and_chromium_run(&tx, |_| ())
            .await
            .unwrap();
    }
}
