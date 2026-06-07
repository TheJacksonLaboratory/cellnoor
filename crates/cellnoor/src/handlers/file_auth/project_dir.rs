use std::convert::identity;

use axum::extract::{Path, State};
use nonempty::NonemptyString;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    db::{self, SqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[schemars(inline)]
pub struct ProjectDir {
    project_name: NonemptyString,
    _file_path: Option<String>,
}

#[axum::debug_handler]
pub async fn authorize_project_dir_access(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ProjectDir {
        project_name,
        _file_path,
    }): Path<ProjectDir>,
) -> Result<(), Error> {
    // If we know the user is staff without hitting the db (via the JWT), just
    // return OK
    if user.is_staff().is_some_and(identity) {
        return Ok(());
    }

    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    project_exists(tx, &project_name)
        .await?
        .then_some(())
        .ok_or(ErrorInner::ResourceNotFound.into())
}

async fn project_exists(
    tx: db::Transaction<'_>,
    project_name: &NonemptyString,
) -> Result<bool, ErrorInner> {
    static SELECT_DATASET: SqlBuilder =
        SqlBuilder::new("select exists (select 1 from project where name = $1)");

    Ok(tx
        .query_one_into(&SELECT_DATASET.finish_with_params(vec![project_name]))
        .await?)
}
