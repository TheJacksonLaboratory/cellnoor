use axum::extract::{Path, State};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::file_auth::FileAuthError,
    state::AppState,
};

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[schemars(inline)]
pub struct ProjectDir {
    project_name: String,
    _file_path: Option<String>,
}

pub async fn authorize_project_dir_access(
    state: State<AppState>,
    user: AuthUser,
    Path(ProjectDir {
        project_name,
        _file_path,
    }): Path<ProjectDir>,
) -> Result<(), FileAuthError> {
    tracing::debug!(
        %project_name,
        file_path = _file_path.unwrap_or_default()
    );

    // If we know the user is staff, just return early
    if user.is_staff() {
        return Ok(());
    }

    let mut client = state.db_client(user).await.map_err(DbError::from)?;
    let tx = client.begin().await.map_err(DbError::from)?;

    project_exists(tx, &project_name)
        .await?
        .then_some(())
        .ok_or(FileAuthError::ProjectAccessDenied)
}

async fn project_exists(tx: db::Transaction<'_>, project_name: &str) -> Result<bool, DbError> {
    static SELECT_DATASET: &str = "select exists (select 1 from project where name = $1)";

    tx.query_one_into(&Sql::new(SELECT_DATASET, vec![&project_name]))
        .await
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            file_auth::project_dir::project_exists,
            people::create::test::insert_test_person_and_institution,
            projects::create::test::insert_test_project,
        },
        state::dev_util::{db_client_as_admin, db_client_as_user},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn only_project_member_sees_project() {
        let mut admin = db_client_as_admin().await;
        let tx = admin.begin().await.unwrap();

        let (_, project) = insert_test_project(&tx, |_| ()).await.unwrap();
        let (_, outsider) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();

        // Commit so the data persists for the clients below
        tx.commit().await.unwrap();

        let project_name = project.record.project.name.as_ref().to_owned();
        let member_id = project.record.members[0];

        let mut client = db_client_as_user(member_id).await;
        let tx = client.begin().await.unwrap();

        assert!(project_exists(tx, &project_name).await.unwrap());

        let mut client = db_client_as_user(outsider.record.id).await;
        let tx = client.begin().await.unwrap();

        assert!(!project_exists(tx, &project_name).await.unwrap());
    }
}
