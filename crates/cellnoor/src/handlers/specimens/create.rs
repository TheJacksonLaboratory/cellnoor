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

    use cellnoor_types::specimen::{
        Species, Specimen,
        creation::{NewSpecimen, NewSpecimenCommonFields, block::NewBlock},
        measurement::{NewSpecimenMeasurement, SpecimenMeasurementData},
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
            people::create::test::insert_test_person_and_institution,
            projects::create::test::insert_test_project, specimens::create::insert_specimen,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_specimen_and_project<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> Specimen
    where
        F: Fn(NewSpecimen) -> NewSpecimen,
    {
        let person_id = *insert_test_person_and_institution(tx, identity)
            .await
            .record
            .id;
        let project = insert_test_project(tx, identity).await;

        let mut new = NewSpecimen::Block(NewBlock::CarboxymethylCellulose {
            inner: NewSpecimenCommonFields {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                name: "specimen".to_nonempty_string(),
                submitted_by: person_id,
                received_at: Timestamp::now(),
                project_id: *project.record().id,
                species: Species::MusMusculus,
                host_species: None,
                returned_by: None,
                returned_at: None,
                tissue: "tissue".to_nonempty_string(),
                additional_data: None,
                measurements: vec![NewSpecimenMeasurement {
                    measured_by: person_id,
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

        insert_specimen(tx, new).await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_specimen_and_project(&tx, identity).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_invalid_timestamp() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Use the underlying insert directly (rather than the unwrapping helper) so
        // we can inspect the error.
        let project = insert_test_project(&tx, identity).await;
        let person = insert_test_person_and_institution(&tx, identity).await;
        let person_id = *person.record.id;

        let new = NewSpecimen::Block(NewBlock::CarboxymethylCellulose {
            inner: NewSpecimenCommonFields {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                name: "specimen".to_nonempty_string(),
                submitted_by: person_id,
                received_at: Timestamp::now() - SignedDuration::from_hours(48),
                project_id: *project.record().id,
                species: Species::MusMusculus,
                host_species: None,
                returned_by: None,
                returned_at: None,
                tissue: "tissue".to_nonempty_string(),
                additional_data: None,
                measurements: vec![NewSpecimenMeasurement {
                    measured_by: person_id,
                    measured_at: Timestamp::now(),
                    data: Json(SpecimenMeasurementData::Rin {
                        instrument_name: None,
                        value: PositiveBoundedF32::new(5.0).unwrap(),
                    }),
                }],
            },
            fixative: None,
        });

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
