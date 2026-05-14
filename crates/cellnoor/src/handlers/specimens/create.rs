use axum::{Json, extract::State};
use cellnoor_types::specimen::{
    Specimen,
    creation::{NewSpecimen, NewSpecimenRecord},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, FieldValuePairs, ToFieldListPlaceholdersParams},
    },
    error::{Error, ErrorInner},
    handlers::specimens::{
        measurements::create::insert_specimen_measurement, show::select_specimen_by_id,
    },
    state::AppState,
};

pub async fn create_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<Specimen>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_specimen(&tx, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_specimen(
    tx: &db::Transaction<'_>,
    record: NewSpecimen,
) -> Result<Specimen, ErrorInner> {
    let (record, measurements) = record.split_for_insertion();

    let id = insert_specimen_record(tx, &record).await?;

    futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_specimen_measurement(tx, id, m)),
    )
    .await?;

    select_specimen_by_id(tx, id).await
}

async fn insert_specimen_record(
    tx: &db::Transaction<'_>,
    new_record: &NewSpecimenRecord,
) -> Result<Uuid, ErrorInner> {
    let fields = new_record.as_field_value_pairs();

    let (field_list, placeholders, params) = fields.to_field_list_and_placeholders_and_params();

    let id = tx
        .query_one_into(
            &format!("insert into specimen {field_list} values {placeholders} returning id"),
            &params,
        )
        .await?;

    Ok(id)
}

impl AsFieldValuePairs<15> for NewSpecimenRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, 15> {
        let Self {
            id: _,
            readable_id,
            name,
            submitted_by,
            received_at,
            project_id,
            species,
            host_species,
            returned_by,
            returned_at,
            tissue,
            additional_data,
            type_,
            embedded_in,
            fixative,
            thermal_preservation_method,
        } = self;

        [
            ("readable_id", readable_id),
            ("name", name),
            ("submitted_by", submitted_by),
            ("received_at", received_at),
            ("project_id", project_id),
            ("species", species),
            ("host_species", host_species),
            ("returned_by", returned_by),
            ("returned_at", returned_at),
            ("tissue", tissue),
            ("additional_data", additional_data),
            ("type", type_),
            ("embedded_in", embedded_in),
            ("fixative", fixative),
            ("thermal_preservation_method", thermal_preservation_method),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use std::convert::identity;

    use cellnoor_types::{
        project::{Project, SavedProjectRecordDetailed},
        specimen::{
            SavedSpecimenRecord, Species, Specimen,
            creation::{NewSpecimen, NewSpecimenCommonFields, block::NewBlock},
            measurement::{NewSpecimenMeasurement, SpecimenMeasurementData},
        },
    };
    use jiff::{SignedDuration, Timestamp};
    use positive::PositiveBoundedF32;
    use postgres_types::Json;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            projects::create::test::insert_test_project, specimens::create::insert_specimen,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_specimen_and_project<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> (NewSpecimen, Specimen)
    where
        F: Fn(NewSpecimen) -> NewSpecimen,
    {
        let (
            _,
            Project::Detailed {
                record: SavedProjectRecordDetailed { project, people },
                links: _,
            },
        ) = insert_test_project(tx, identity).await
        else {
            panic!("expected detailed project");
        };

        let mut new = NewSpecimen::Block(NewBlock::CarboxymethylCellulose {
            inner: NewSpecimenCommonFields {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                name: "specimen".to_nonempty_string(),
                submitted_by: people[0],
                received_at: Timestamp::now(),
                project_id: *project.id,
                species: Species::MusMusculus,
                host_species: None,
                returned_by: None,
                returned_at: None,
                tissue: "tissue".to_nonempty_string(),
                additional_data: None,
                measurements: vec![NewSpecimenMeasurement {
                    measured_by: people[0],
                    measured_at: Timestamp::now(),
                    data: Json(SpecimenMeasurementData::Rin {
                        instrument_name: None,
                        value: PositiveBoundedF32::new(5.0).unwrap(),
                    }),
                }],
            },
            fixative: None,
        });

        new = modify(new);

        let inserted = insert_specimen(tx, new.clone()).await.unwrap();
        (new, inserted)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            input,
            Specimen::Detailed {
                record: output_record,
                project: output_project,
                measurements: output_measurements,
                links: _,
            },
        ) = insert_test_specimen_and_project(&tx, identity).await
        else {
            panic!("expected Specimen::Detailed");
        };

        let (input_record, input_measurements) = input.split_for_insertion();

        let expected_record = SavedSpecimenRecord {
            id: output_record.id,
            readable_id: input_record.readable_id,
            name: input_record.name,
            submitted_by: input_record.submitted_by,
            project_id: input_record.project_id,
            received_at: input_record.received_at,
            species: input_record.species,
            host_species: input_record.host_species,
            returned_at: input_record.returned_at,
            returned_by: input_record.returned_by,
            type_: input_record.type_,
            embedded_in: input_record.embedded_in,
            fixative: input_record.fixative,
            thermal_preservation_method: input_record.thermal_preservation_method,
            tissue: input_record.tissue,
            additional_data: input_record.additional_data,
        };

        assert_eq!(output_record, expected_record);
        assert_eq!(*output_project.record().id, expected_record.project_id);

        // Measurements: ids are auto-generated, but everything else should match
        // what we asked to insert.
        assert_eq!(output_measurements.len(), input_measurements.len());
        for (out, inp) in output_measurements.iter().zip(input_measurements.iter()) {
            assert_eq!(out.specimen_id, *output_record.id);
            assert_eq!(out.measured_by, inp.measured_by);
            assert_eq!(out.measured_at, inp.measured_at);
            assert_eq!(out.data, inp.data);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_invalid_timestamp() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Just use the new specimen returned by this test helper
        let (mut new, _) = insert_test_specimen_and_project(&tx, identity).await;
        let inner = new.inner_mut();
        inner.readable_id = Uuid::new_v4().to_string().to_nonempty_string();
        inner.received_at = Timestamp::from_second(0).unwrap();

        let error = insert_specimen(&tx, new).await.unwrap_err();

        assert_eq!(
            error,
            ErrorInner::DataConstraint {
                resource: Some("specimen".to_owned()),
                field: Some("received_at".to_owned()),
                message: "received_at cannot be before parent project field started_at".to_owned(),
                detail: None
            }
        );
    }
}
