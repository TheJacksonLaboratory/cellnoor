use axum::{Json, extract::State};
use cellnoor_types::specimen::{
    Specimen,
    creation::{NewSpecimen, NewSpecimenRecord},
};

use crate::{
    auth::AuthUser,
    db::{self, Record, ToRecord},
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

    let id = db::insert_into(tx, "specimen", &record).await?;

    futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_specimen_measurement(tx, id, m)),
    )
    .await?;

    select_specimen_by_id(tx, id).await
}

impl ToRecord<&'static str, 15> for NewSpecimenRecord {
    fn to_record(&self) -> Record<'_, &'static str, 15> {
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
    use cellnoor_types::{
        project::{Project, SavedProjectRecordDetailed},
        specimen::{
            Species, Specimen,
            creation::{NewSpecimen, NewSpecimenCommonFields, block::NewBlock},
            measurement::{NewSpecimenMeasurement, SpecimenMeasurementData},
        },
    };
    use jiff::Timestamp;
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
        mut modify: F,
    ) -> Result<(NewSpecimen, Specimen), ErrorInner>
    where
        F: FnMut(&mut NewSpecimen),
    {
        let (
            _,
            Project::Detailed {
                record: SavedProjectRecordDetailed { project, people },
                links: _,
            },
        ) = insert_test_project(tx, |_| ()).await?
        else {
            panic!("expected Project::Detailed");
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
            sp.inner_mut().received_at = Timestamp::from_second(0).unwrap()
        })
        .await
        .unwrap_err();

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
