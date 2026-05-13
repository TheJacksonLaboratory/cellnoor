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
