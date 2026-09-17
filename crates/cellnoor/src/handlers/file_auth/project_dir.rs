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
