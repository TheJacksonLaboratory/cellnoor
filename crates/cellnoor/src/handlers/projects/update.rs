use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::project::{NewProject, Project};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, ToUpdateClause},
    },
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        projects::{access::add_person::insert_project_accesses, show::select_project_by_id},
    },
    state::AppState,
};

pub async fn update_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(project): Json<NewProject>,
) -> Result<Json<Project>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_project_by_id(&tx, id, &project).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    NewProject { record, people }: &NewProject,
) -> Result<Project, ErrorInner> {
    let fields = record.as_field_value_pairs();
    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update project set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    insert_project_accesses(tx, id, people).await?;

    select_project_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        project::{NewProject, NewProjectRecord, SavedProjectRecord},
    };
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
        handlers::projects::{create::test::insert_test_project, update::update_project_by_id},
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let original_project = insert_test_project(&tx, identity).await;

        let new_name = Uuid::new_v4().to_string().to_nonempty_string();
        let new_started_at = Timestamp::now();
        let new_ended_at = Timestamp::now();

        let new_data = NewProject {
            record: NewProjectRecord {
                id: NoId {},
                name: new_name.clone(),
                started_at: new_started_at,
                ended_at: new_ended_at,
            },
            people: vec![],
        };

        let updated = update_project_by_id(&tx, *original_project.record().id, &new_data)
            .await
            .unwrap();

        assert_eq!(
            updated.record(),
            &SavedProjectRecord {
                id: original_project.record().id,
                name: new_name,
                started_at: new_started_at,
                ended_at: new_ended_at
            }
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let new_data = NewProject {
            record: NewProjectRecord {
                id: NoId {},
                name: "missing".to_nonempty_string(),
                started_at: Timestamp::now(),
                ended_at: Timestamp::now(),
            },
            people: vec![],
        };

        let error = update_project_by_id(&tx, Uuid::new_v4(), &new_data)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
