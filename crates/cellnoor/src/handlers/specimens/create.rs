use axum::{Json, extract::State};
use cellnoor_types::specimen::{
    NewSpecimenRecord, SpecimenDetailed, SpecimenField, creation::NewSpecimen,
};

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::specimens::{
        measurements::create::insert_specimen_measurements, show::select_specimen_by_id,
        split_new_specimen_for_insertion::split_new_specimen_for_insertion,
    },
    state::AppState,
};

pub async fn create_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<SpecimenDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_specimen(&tx, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_specimen(
    tx: &db::Transaction<'_>,
    record: NewSpecimen,
) -> Result<SpecimenDetailed, ErrorInner> {
    let (record, measurements) = split_new_specimen_for_insertion(record);

    let id = tx.insert_returning_id(&record).await?;

    insert_specimen_measurements(tx, id, &measurements).await?;

    select_specimen_by_id(tx, id).await
}

impl Insert for NewSpecimenRecord {
    type Field = SpecimenField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use SpecimenField::*;

        let Self {
            id: _,
            readable_id,
            name,
            submitted_by,
            received_at,
            project_id,
            // Populated by a database trigger
            project_started_at: _,
            project_ended_at: _,
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

        vec![
            (ReadableId, readable_id),
            (Name, name),
            (SubmittedBy, submitted_by),
            (ReceivedAt, received_at),
            (ProjectId, project_id),
            (Species, species),
            (HostSpecies, host_species),
            (ReturnedBy, returned_by),
            (ReturnedAt, returned_at),
            (Tissue, tissue),
            (AdditionalData, additional_data),
            (Type, type_),
            (EmbeddedIn, embedded_in),
            (Fixative, fixative),
            (ThermalPreservationMethod, thermal_preservation_method),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        project::SavedProjectRecordDetailed,
        specimen::{
            Species, SpecimenDetailed,
            creation::{FlashFreezing, NewSpecimen, SpecimenVariableFields, block::BlockFields},
            measurement::{
                NewSpecimenMeasurement, SpecimenMeasurementData, SpecimenMeasurementQuantity,
            },
        },
    };
    use jiff::Timestamp;
    use positive::PositiveBoundedF32;
    use postgres_types::Json;
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
        mut modify: F,
    ) -> Result<(NewSpecimen, SpecimenDetailed), ErrorInner>
    where
        F: FnMut(&mut NewSpecimen),
    {
        let (_, inserted_project) = insert_test_project(tx, |_| ()).await?;
        let SavedProjectRecordDetailed { project, members } = inserted_project.record;
        let mut new = NewSpecimen {
            readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            submitted_by: members[0],
            received_at: Timestamp::now(),
            project_id: project.id,
            species: Species::MusMusculus,
            host_species: None,
            returned_by: None,
            returned_at: None,
            tissue: "tissue".to_nonempty_string(),
            additional_data: None,
            measurements: vec![NewSpecimenMeasurement {
                measured_by: members[0],
                measured_at: Timestamp::now(),
                data: Json(SpecimenMeasurementData {
                    instrument_name: None,
                    quantity: SpecimenMeasurementQuantity::Rin {
                        value: PositiveBoundedF32::new(5.0).unwrap(),
                    },
                }),
            }],
            variable_fields: SpecimenVariableFields::Block(BlockFields::CarboxymethylCellulose {
                fixative: None,
                thermal_preservation_method: FlashFreezing::FlashFreezing,
            }),
        };

        modify(&mut new);

        let inserted = insert_specimen(tx, new.clone()).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_invalid_timestamp() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = insert_test_specimen_and_project(&tx, |sp| {
            sp.received_at = Timestamp::from_second(0).unwrap()
        })
        .await
        .unwrap_err();

        std::assert_matches!(
            error,
            ErrorInner::DataConstraint {
                resource: Some(_),
                field: None,
                message: _,
                detail: None,
            }
        );
    }
}
