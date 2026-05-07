use axum::{Json, extract::State};
use cellnoor_types::specimen::{
    NewSpecimen, Specimen, SpecimenCommonFields, SpecimenVariableFields,
    measurement::NewSpecimenMeasurement,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser, db, error::Error, handlers::specimens::show::select_specimen_by_id,
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
    insert_specimen_inner(tx, &record.split_for_insertion()).await
}

async fn insert_specimen_inner(
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
            measurements,
        },
        SpecimenVariableFields {
            type_,
            embedded_in,
            fixative,
            thermal_preservation_method,
        },
    ): &(SpecimenCommonFields, SpecimenVariableFields),
) -> Result<Specimen, Error> {
    let fields = [
        "readable_id",
        "name",
        "submitted_by",
        "project_id",
        "received_at",
        "species",
        "host_species",
        "returned_at",
        "returned_by",
        "type",
        "embedded_in",
        "fixative",
        "thermal_preservation_method",
        "tissue",
        "additional_data",
    ];

    let mut param_expression = String::with_capacity(32);
    param_expression.push('(');
    for (i, _) in fields.iter().enumerate() {
        param_expression.push_str(&format!("${}", i + 1));
        if i != fields.len() - 1 {
            param_expression.push(',');
        }
    }
    param_expression.push(')');

    let field_expression = format!("({})", fields.join(", "));

    let id = tx
        .query_one_into(
            &format!(
                "insert into specimen {field_expression} values {param_expression} returning id"
            ),
            &[
                readable_id,
                name,
                submitted_by,
                project_id,
                received_at,
                species,
                host_species,
                returned_at,
                returned_by,
                type_,
                embedded_in,
                fixative,
                thermal_preservation_method,
                tissue,
                additional_data,
            ],
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
                received_at: Timestamp::now() + SignedDuration::new(60 * 60 * 24, 0),
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
