use axum::{Json, extract::State};
use cellnoor_types::{
    Relation,
    nonempty::NonemptyString,
    specimen::{
        Fixative, SavedSpecimenRecord, SpecimenDetailed, SpecimenField, SpecimenType,
        ThermalPreservationMethod,
        creation::{NewSpecimen, SpecimenOrigin, block::BlockEmbeddingMatrix},
    },
};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::specimens::{
        convert_specimen_fields_to_record::convert_specimen_fields_to_record,
        measurements::create::insert_specimen_measurements, show::select_specimen_by_id,
    },
    state::AppState,
};

pub async fn create_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<NewSpecimen>,
) -> Result<Json<SpecimenDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| insert_specimen(tx, record).await)
        .await
}

pub(in super::super) async fn insert_specimen(
    tx: &db::Transaction<'_>,
    NewSpecimen {
        simple,
        measurements,
    }: NewSpecimen,
) -> Result<SpecimenDetailed, DbError> {
    let id = tx
        .insert_returning_id(&convert_specimen_fields_to_record(simple))
        .await?;

    insert_specimen_measurements(tx, id, &measurements).await?;

    select_specimen_by_id(tx, id).await
}

pub(super) struct NewSpecimenRecord {
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub origin: SpecimenOrigin,
    pub received_or_created_at: Timestamp,
    pub returned_at: Option<Timestamp>,
    pub returned_by: Option<Uuid>,
    pub type_: SpecimenType,
    pub embedded_in: Option<BlockEmbeddingMatrix>,
    pub fixative: Option<Fixative>,
    pub thermal_preservation_method: Option<ThermalPreservationMethod>,
    pub additional_data: Option<serde_json::Value>,
}

impl Relation for NewSpecimenRecord {
    const NAME: &'static str = <SavedSpecimenRecord as Relation>::NAME;
}

impl Insert for NewSpecimenRecord {
    type Field = SpecimenField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use SpecimenField::*;

        let Self {
            readable_id,
            name,
            origin,
            received_or_created_at,
            returned_by,
            returned_at,
            additional_data,
            type_,
            embedded_in,
            fixative,
            thermal_preservation_method,
        } = self;

        let mut fields: FieldValues<'_, Self::Field> = vec![
            (ReadableId, readable_id),
            (Name, name),
            (ReceivedOrCreatedAt, received_or_created_at),
            (ReturnedBy, returned_by),
            (ReturnedAt, returned_at),
            (AdditionalData, additional_data),
            (Type, type_),
            (EmbeddedIn, embedded_in),
            (Fixative, fixative),
            (ThermalPreservationMethod, thermal_preservation_method),
        ];

        // Each variant writes null to the other variant's column so that an
        // update can turn a primary specimen into a derived one and vice versa
        match origin {
            SpecimenOrigin::Primary {
                submitted_by,
                project_id,
                species,
                host_species,
                tissue,
            } => {
                fields.push((SubmittedBy, submitted_by));
                fields.push((ParentSpecimenId, &None::<Uuid>));
                fields.push((ProjectId, project_id));
                fields.push((Species, species));
                fields.push((HostSpecies, host_species));
                fields.push((Tissue, tissue));
            }
            // The project, species, host species, and tissue are copied from
            // the parent by a database trigger
            SpecimenOrigin::Derived { parent_specimen_id } => {
                fields.push((ParentSpecimenId, parent_specimen_id));
                fields.push((SubmittedBy, &None::<Uuid>));
            }
        }

        fields
    }
}

#[cfg(any(test, feature = "dev"))]
pub async fn insert_test_specimen_and_project<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewSpecimen, SpecimenDetailed), DbError>
where
    F: FnMut(&mut NewSpecimen),
{
    use cellnoor_types::{
        positive::PositiveBoundedF32,
        project::SavedProjectRecordDetailed,
        specimen::{
            Species,
            creation::{
                FlashFreezing, SpecimenSimpleFields, SpecimenVariableFields, block::BlockFields,
            },
            measurement::{
                NewSpecimenMeasurement, SpecimenMeasurementData, SpecimenMeasurementQuantity,
            },
        },
    };
    use jiff::Timestamp;
    use postgres_types::Json;

    use crate::{
        handlers::projects::create::insert_test_project, state::dev_util::ToNonemptyString,
    };

    let (_, inserted_project) = insert_test_project(tx, |_| ()).await?;
    let SavedProjectRecordDetailed { project, members } = inserted_project.record;
    let mut new = NewSpecimen {
        simple: SpecimenSimpleFields {
            readable_id: crate::db::dummy_data::random_name_for("26SP"),
            name: crate::db::dummy_data::random_name_for("specimen"),
            received_or_created_at: Timestamp::now(),
            returned_by: None,
            returned_at: None,
            additional_data: None,
            origin: SpecimenOrigin::Primary {
                submitted_by: members[0],
                project_id: project.id,
                species: Species::MusMusculus,
                host_species: None,
                tissue: "tissue".to_nonempty_string(),
            },
            variable_fields: SpecimenVariableFields::Block(BlockFields::CarboxymethylCellulose {
                fixative: None,
                thermal_preservation_method: FlashFreezing::FlashFreezing,
            }),
        },
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
    };

    modify(&mut new);

    let inserted = insert_specimen(tx, new.clone()).await?;
    Ok((new, inserted))
}

#[cfg(test)]
pub mod test {

    use cellnoor_types::specimen::{Species, creation::SpecimenOrigin};
    use jiff::Timestamp;

    use super::insert_test_specimen_and_project;
    use crate::{db::DbError, state::dev_util::db_client_as_admin};

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
            sp.simple.received_or_created_at = Timestamp::from_second(0).unwrap()
        })
        .await
        .unwrap_err();

        std::assert_matches!(
            error,
            DbError::DataConstraint {
                resource: Some(_),
                field: None,
                message: _,
                detail: None,
            }
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn derived_specimen_inherits_from_parent() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, parent) = insert_test_specimen_and_project(&tx, |sp| {
            if let SpecimenOrigin::Primary { host_species, .. } = &mut sp.simple.origin {
                *host_species = Some(Species::HomoSapiens);
            }
        })
        .await
        .unwrap();
        let (_, child) = insert_test_specimen_and_project(&tx, |sp| {
            sp.simple.origin = SpecimenOrigin::Derived {
                parent_specimen_id: *parent.record.id,
            };
        })
        .await
        .unwrap();

        let (parent, child) = (parent.record, child.record);
        assert_eq!(child.submitted_by, None);
        assert_eq!(
            (child.project_id, child.species, child.host_species, child.tissue),
            (parent.project_id, parent.species, parent.host_species, parent.tissue)
        );
    }
}
