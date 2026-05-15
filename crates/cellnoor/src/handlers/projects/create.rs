use axum::{Json, extract::State};
use cellnoor_types::project::{NewProject, NewProjectRecord, Project};

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, FieldValuePairs, ToFieldListPlaceholdersParams},
    },
    error::{Error, ErrorInner},
    handlers::projects::{access::add_person::insert_project_accesses, show::select_project_by_id},
    state::AppState,
};

pub async fn create_project(
    State(state): State<AppState>,
    user: AuthUser,
    Json(project): Json<NewProject>,
) -> Result<Json<Project>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_project(&tx, &project).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_project(
    tx: &db::Transaction<'_>,
    NewProject { record, people }: &NewProject,
) -> Result<Project, ErrorInner> {
    let fields = record.as_field_value_pairs();
    let (field_list, placeholders, params) = fields.to_field_list_and_placeholders_and_params();

    let id = tx
        .query_one_into(
            &format!("insert into project {field_list} values {placeholders} returning id"),
            &params,
        )
        .await?;

    insert_project_accesses(&tx, id, &people).await?;

    select_project_by_id(tx, id).await
}

impl AsFieldValuePairs<3> for NewProjectRecord {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, 3> {
        let Self {
            id: _,
            name,
            started_at,
            ended_at,
        } = self;

        [
            ("name", name),
            ("started_at", started_at),
            ("ended_at", ended_at),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        project::{
            NewProject, NewProjectRecord, Project, SavedProjectRecord, SavedProjectRecordDetailed,
        },
    };
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        handlers::{
            people::create::test::insert_test_person_and_institution,
            projects::create::insert_project,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_project<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> (NewProject, Project)
    where
        F: FnMut(&mut NewProject),
    {
        let (_, person) = insert_test_person_and_institution(tx, identity).await;
        let person_id = *person.record.id;

        let mut new = NewProject {
            record: NewProjectRecord {
                id: NoId {},
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                started_at: Timestamp::now(),
                ended_at: Timestamp::now(),
            },
            people: vec![person_id],
        };

        modify(&mut new);

        let inserted = insert_project(tx, &new).await.unwrap();
        (new, inserted)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            NewProject {
                record: input_record,
                people: input_people,
            },
            Project::Detailed {
                record:
                    SavedProjectRecordDetailed {
                        project: output_record,
                        people: output_people,
                    },
                links: _,
            },
        ) = insert_test_project(&tx, identity).await
        else {
            panic!("expected Project::Detailed");
        };

        let expected_record = SavedProjectRecord {
            id: output_record.id,
            name: input_record.name,
            started_at: input_record.started_at,
            ended_at: input_record.ended_at,
        };

        assert_eq!(output_record, expected_record);
        assert_eq!(output_people, input_people);
    }
}
