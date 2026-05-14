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
        project::{NewProject, NewProjectRecord, Project},
    };
    use jiff::Timestamp;
    use uuid::Uuid;

    use crate::{
        db,
        handlers::projects::create::insert_project,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_project<F>(tx: &db::Transaction<'_>, modify: F) -> Project
    where
        F: Fn(NewProject) -> NewProject,
    {
        let mut new = NewProject {
            record: NewProjectRecord {
                id: NoId {},
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                started_at: Timestamp::now(),
                ended_at: Timestamp::now(),
            },
            people: vec![],
        };

        new = modify(new);

        insert_project(tx, &new).await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_project(&tx, identity).await;
    }
}
