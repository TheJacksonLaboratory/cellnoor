use axum::{Json, extract::State};
use cellnoor_types::{
    institution::{Institution, NewInstitution},
    project::{NewProject, Project},
    specimen::{NewBlock, NewSpecimen, Specimen, SpecimenCommonFields, SpecimenVariableFields},
};

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{JunctionTable, insert_many_to_many},
    },
    error::Error,
    handlers::projects::show::select_project_by_id,
    state::AppState,
};

pub async fn create_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_specimen(&tx, record).await.map(Json);

    tx.commit().await?;

    response
}

pub async fn insert_specimen(
    tx: &db::Transaction<'_>,
    record: NewSpecimen,
) -> Result<(), crate::error::Error> {
    let NewSpecimen::Block(b) = record else {
        unreachable!();
    };
    insert_specimen_inner(tx, &b.split_for_insertion()).await?;

    Ok(())
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
        },
        SpecimenVariableFields {
            type_,
            embedded_in,
            fixative,
            thermal_preservation_method,
        },
    ): &(SpecimenCommonFields, SpecimenVariableFields),
) -> Result<(), Error> {
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

    tx.execute(
        &format!("insert into specimen {field_expression} values {param_expression}"),
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

    Ok(())
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        SimpleStringOperator, StringOperator,
        institution::{InstitutionPredicate, InstitutionQuery},
        person::{PersonPredicate, PersonQuery},
        project::{NewProject, Project, ProjectQuery, ProjectRecordDetailed},
        specimen::{NewBlock, NewSpecimen, Species, SpecimenCommonFields},
    };
    use jiff::{SignedDuration, Timestamp};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::{Error, ErrorInner},
        handlers::{
            projects::{
                create::{insert_project, test::new_project},
                index::select_projects,
            },
            specimens::create::insert_specimen,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub fn new_specimen(readable_id: &str, project_id: Uuid) -> NewSpecimen {
        NewSpecimen::Block(NewBlock::CarboxymethylCellulose {
            inner: SpecimenCommonFields {
                readable_id: readable_id.to_nonempty_string(),
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
            },
            fixative: None,
        })
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let project = insert_project(&tx, &new_project()).await.unwrap();

        let new = new_specimen("SP1", project.id());
        insert_specimen(&tx, new).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_invalid_timestamp() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let project = insert_project(&tx, &new_project()).await.unwrap();

        let mut new = new_specimen("SP1", project.id());
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
