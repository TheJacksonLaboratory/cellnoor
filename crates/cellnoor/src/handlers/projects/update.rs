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
        util::{FieldValuePairs, ToUpdateClause},
    },
    error::Error,
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

    let response = update_project_by_id(&tx, id, &project).await.map(Json);

    tx.commit().await?;

    response
}

pub async fn update_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    NewProject {
        name,
        started_at,
        ended_at,
        people,
    }: &NewProject,
) -> Result<Project, Error> {
    let fields: FieldValuePairs<_> = [
        ("name", name),
        ("started_at", started_at),
        ("ended_at", ended_at),
    ];

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update project set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(Error::resource_not_found());
    }

    insert_project_accesses(tx, id, people).await?;

    select_project_by_id(tx, id).await
}
