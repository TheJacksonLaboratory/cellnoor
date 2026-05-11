use axum::{Json, extract::State};
use cellnoor_types::project::{NewProject, Project};

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToFieldListPlaceholdersParams},
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
    NewProject {
        name,
        started_at,
        ended_at,
        people,
    }: &NewProject,
) -> Result<Project, ErrorInner> {
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
        error::ErrorInner,
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
        let tx = client.begin().await.unwrap();

        let new = new_project();
        let id = insert(&tx, &new).await;
        select(&tx, &new, id).await;
        update(&tx, id).await;
        delete(&tx, id).await;
    }

    async fn insert(tx: &db::Transaction<'_>, new: &NewProject) -> Uuid {
        let inserted = insert_project(&tx, new).await.unwrap();
        let id = inserted.record().id;

        id
    }

    async fn select(tx: &db::Transaction<'_>, new: &NewProject, id: Uuid) {
        let query = ProjectQuery::from_filter(
            ProjectPredicate::Name(SimpleStringOperator::Eq(new.name.clone().into()).into()),
            false,
        );
        let selected = select_projects(&tx, &query).await.unwrap();

        assert_eq!(selected[0].record().id, id);
    }

    async fn update(tx: &db::Transaction<'_>, id: Uuid) {
        let new_data = new_project();
        let updated = update_project_by_id(&tx, id, &new_data).await.unwrap();

        assert_eq!(updated.record().id, id);
        assert_eq!(updated.record().name, new_data.name);
    }

    async fn delete(tx: &db::Transaction<'_>, id: Uuid) {
        delete_project_by_id(&tx, id).await.unwrap();

        // Verify the project no longer exists
        let error = select_project_by_id(&tx, id).await.unwrap_err();
        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
