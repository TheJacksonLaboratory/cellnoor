use std::str::FromStr;

use axum::{Json, extract::State};
use cellnoor_types::cdna::{
    CdnaDetailed, CdnaField,
    creation::{CdnaSimpleFields, LibraryType, NewCdna, NewCdnaCommonFields},
};
use positive::PositiveI32;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::cdna::{measurements::create::insert_cdna_measurement, show::select_cdna_by_id},
    state::AppState,
};

pub async fn create_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewCdna>,
) -> Result<Json<CdnaDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_cdna(&tx, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_cdna(tx: &db::Transaction<'_>, new: &NewCdna) -> Result<CdnaDetailed, ErrorInner> {
    let generic = NewCdnaGeneric::from_new_cdna(&new);
    let id = db::insert_into(tx, "cdna", &generic).await?;

    let measurement_insertions = futures::future::try_join_all(
        generic
            .common
            .measurements
            .iter()
            .map(|m| insert_cdna_measurement(tx, id, m)),
    );

    tokio::try_join!(
        insert_cdna_preparers(tx, id, generic.common.preparers.as_ref()),
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

    futures::future::try_join_all(
        preparers
            .iter()
            .map(|p| db::insert_into_no_returning(tx, "cdna_preparer", p)),
    )
    .await?;

    Ok(())
}

struct NewCdnaPreparer {
    cdna_id: Uuid,
    prepared_by: Uuid,
}

impl AsFieldValuePairs<&'static str, 2> for NewCdnaPreparer {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 2> {
        let Self {
            cdna_id,
            prepared_by,
        } = self;

        [("cdna_id", cdna_id), ("prepared_by", prepared_by)]
    }
}

struct NewCdnaGeneric<'a> {
    common: &'a NewCdnaCommonFields,
    gem_well_id: Option<Uuid>,
    library_type: LibraryType,
    n_amplification_cycles: Option<PositiveI32>,
}

impl<'a> NewCdnaGeneric<'a> {
    fn from_new_cdna(cdna: &'a NewCdna) -> Self {
        let library_type = cdna.into();

        let (common, gem_well_id, n_amplification_cycles) = match cdna {
            NewCdna::AntibodyCapture(common)
            | NewCdna::AntigenCapture(common)
            | NewCdna::ChromatinAccessibility(common)
            | NewCdna::CrisprGuideCapture(common)
            | NewCdna::Custom(common)
            | NewCdna::MultiplexingCapture(common)
            | NewCdna::Vdj(common)
            | NewCdna::VdjB(common)
            | NewCdna::VdjT(common)
            | NewCdna::VdjTGd(common) => (&common.common, Some(common.gem_well_id), None),
            NewCdna::GeneExpression {
                common,
                n_amplification_cycles,
            } => (
                &common.common,
                Some(common.gem_well_id),
                Some(*n_amplification_cycles),
            ),
        };

        Self {
            common,
            gem_well_id,
            library_type,
            n_amplification_cycles,
        }
    }
}

impl AsFieldValuePairs<CdnaField, 3> for CdnaSimpleFields {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, CdnaField, 3> {
        use CdnaField::*;

        let Self {
            readable_id,
            prepared_at,
            additional_data,
        } = self;

        [
            (ReadableId, readable_id),
            (PreparedAt, prepared_at),
            (AdditionalData, additional_data),
        ]
    }
}

impl AsFieldValuePairs<CdnaField, 6> for NewCdnaGeneric<'_> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, CdnaField, 6> {
        use CdnaField::*;

        let Self {
            common:
                NewCdnaCommonFields {
                    record,
                    measurements: _,
                    preparers: _,
                },
            gem_well_id,
            library_type,
            n_amplification_cycles,
        } = self;

        // Preallocate an array and then just combine
        let mut fields: FieldValuePairs<'_, CdnaField, 6> = [(ReadableId, &""); 6];

        fields[..3].copy_from_slice(&record.as_field_value_pairs());

        fields[3] = (LibraryType, library_type);
        fields[4] = (GemWellId, gem_well_id);
        fields[5] = (NAmplificationCycles, n_amplification_cycles);

        fields
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        cdna::{
            CdnaDetailed,
            creation::{
                CdnaSimpleFields, NewCdna, NewCdnaCommonFields, NewChromiumCdnaCommonFields,
            },
        },
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

        let mut new = NewCdna::GeneExpression {
            common: NewChromiumCdnaCommonFields {
                common: NewCdnaCommonFields {
                    record: CdnaSimpleFields {
                        readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                        prepared_at,
                        additional_data: None,
                    },
                    measurements: vec![NewNucleicAcidMeasurement {
                        measured_by: person_id,
                        measured_at: Timestamp::now(),
                        data: Json(NucleicAcidMeasurementData::Fluorometric {
                            instrument_name: "Qubit".to_nonempty_string(),
                            concentration: Concentration {
                                value: PositiveI32::new(50).unwrap(),
                                numerator_unit: Nanogram::Nanogram,
                                denominator_unit: Microliter::Microliter,
                            },
                        }),
                    }],
                    preparers: nonempty::NonemptyVec::new(vec![person_id]).unwrap(),
                },
                gem_well_id,
            },
            n_amplification_cycles: PositiveI32::new(10).unwrap(),
        };

        modify(&mut new);

        let inserted = insert_cdna(tx, &new).await?;
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
