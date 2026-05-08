use axum::{Json, extract::State};
use cellnoor_types::project::{NewProject, Project};

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToFieldListPlaceholdersParams},
    },
    error::Error,
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

    let response = insert_project(&tx, &project).await.map(Json);

    tx.commit().await?;

    response
}

pub async fn insert_project(
    tx: &db::Transaction<'_>,
    NewProject {
        name,
        started_at,
        ended_at,
        people,
    }: &NewProject,
) -> Result<Project, crate::error::Error> {
    let fields: FieldValuePairs<_> = [
        ("name", name),
        ("started_at", started_at),
        ("ended_at", ended_at),
    ];
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

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        SimpleStringOperator,
        project::{NewProject, ProjectPredicate, ProjectQuery},
    };
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::{Error, ErrorInner},
        handlers::projects::{
            create::insert_project, delete::delete_project_by_id, index::select_projects,
            show::select_project_by_id, update::update_project_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub fn new_project() -> NewProject {
        NewProject {
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            started_at: Timestamp::now(),
            ended_at: Timestamp::now(),
            people: vec![],
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_select_update_delete() {
        let mut client = db_client_as_admin().await;

        let new = new_project();
        let id = insert(&mut client, &new).await;
        select(&mut client, &new, id).await;
        update(&mut client, id).await;
        delete(&mut client, id).await;
    }

    async fn insert(client: &mut db::Client, new: &NewProject) -> Uuid {
        let tx = client.begin().await.unwrap();

        let inserted = insert_project(&tx, new).await.unwrap();
        let id = inserted.record().id;

        tx.commit().await.unwrap();

        id
    }

    async fn select(client: &mut db::Client, new: &NewProject, id: Uuid) {
        let tx = client.begin().await.unwrap();

        // Apply a filter to make sure it works. Note that we fetch the compact
        // representation because we already fetch the detailed one inside of
        // `insert_project`
        let query = ProjectQuery::from_filter(
            ProjectPredicate::Name(SimpleStringOperator::Eq(new.name.clone().into()).into()),
            false,
        );
        let selected = select_projects(&tx, &query).await.unwrap();

        assert_eq!(selected[0].record().id, id);
    }

    async fn update(client: &mut db::Client, id: Uuid) {
        let tx = client.begin().await.unwrap();

        let new_data = new_project();
        let updated = update_project_by_id(&tx, id, &new_data).await.unwrap();

        assert_eq!(updated.record().id, id);
        assert_eq!(updated.record().name, new_data.name);

        tx.commit().await.unwrap();
    }

    async fn delete(client: &mut db::Client, id: Uuid) {
        let tx = client.begin().await.unwrap();
        delete_project_by_id(&tx, id).await.unwrap();
        tx.commit().await.unwrap();

        // Verify the project no longer exists
        let tx = client.begin().await.unwrap();
        let Error { error } = select_project_by_id(&tx, id).await.unwrap_err();
        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
