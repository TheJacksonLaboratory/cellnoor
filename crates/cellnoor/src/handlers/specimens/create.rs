use axum::{Json, extract::State};
use cellnoor_types::specimen::{
    NewSpecimen, Specimen, SpecimenCommonFields, SpecimenVariableFields,
};
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToFieldListPlaceholdersParams},
    },
    error::Error,
    handlers::specimens::show::select_specimen_by_id,
    state::AppState,
};

pub async fn create_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<Specimen>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_specimen(&tx, record).await.map(Json);

    tx.commit().await?;

    response
}

pub async fn insert_specimen(
    tx: &db::Transaction<'_>,
    record: NewSpecimen,
) -> Result<Specimen, crate::error::Error> {
    let ((common_fields, measurements), variable_fields) = record.split_for_insertion();
    insert_specimen_record(tx, &(common_fields, variable_fields)).await
}

async fn insert_specimen_record(
    tx: &db::Transaction<'_>,
    (
        SpecimenCommonFields {
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
            ..
        },
        SpecimenVariableFields {
            type_,
            embedded_in,
            fixative,
            thermal_preservation_method,
        },
    ): &(SpecimenCommonFields, SpecimenVariableFields),
) -> Result<Specimen, Error> {
    let fields: FieldValuePairs<_> = [
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
    ];

    let (field_list, placeholders, params) = fields.to_field_list_placeholders_params();

    let id = tx
        .query_one_into(
            &format!("insert into specimen {field_list} values {placeholders} returning id"),
            &params,
        )
        .await?;

    select_specimen_by_id(tx, id).await
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        UuidOperator,
        specimen::{
            NewBlock, NewSpecimen, Species, Specimen, SpecimenCommonFields, SpecimenPredicate,
            SpecimenQuery,
        },
    };
    use jiff::{SignedDuration, Timestamp};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::{Error, ErrorInner},
        handlers::{
            projects::create::{insert_project, test::new_project},
            specimens::{create::insert_specimen, index::select_specimens},
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub fn new_specimen(project_id: Uuid) -> NewSpecimen {
        NewSpecimen::Block(NewBlock::CarboxymethylCellulose {
            inner: SpecimenCommonFields {
                readable_id: Uuid::new_v4().to_string().to_nonempty_string(),
                name: "specimen".to_nonempty_string(),
                submitted_by: Uuid::nil(),
                received_at: Timestamp::now() + SignedDuration::from_hours(24),
                project_id,
                species: Species::MusMusculus,
                host_species: None,
                returned_by: None,
                returned_at: None,
                tissue: "tissue".to_nonempty_string(),
                additional_data: None,
                measurements: vec![],
            },
            fixative: None,
        })
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_and_select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let project = insert_project(&tx, &new_project()).await.unwrap();

        let new = new_specimen(project.record().id);
        let inserted = insert_specimen(&tx, new).await.unwrap();

        // Apply a filter to make sure it works. Note that we fetch the compact
        // representation because we already fetch the detailed one inside of
        // `insert_project`
        let specimens_from_query = select_specimens(
            &tx,
            &SpecimenQuery::from_filter(
                SpecimenPredicate::Id(UuidOperator::Eq(inserted.record().id)),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(specimens_from_query[0].record(), inserted.record());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_invalid_timestamp() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let project = insert_project(&tx, &new_project()).await.unwrap();

        let mut new = new_specimen(project.record().id);
        match &mut new {
            NewSpecimen::Block(NewBlock::CarboxymethylCellulose { inner, .. }) => {
                inner.received_at = Timestamp::now() - SignedDuration::from_hours(48);
            }
            _ => unreachable!(),
        }

        let Error { error } = insert_specimen(&tx, new).await.unwrap_err();

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
