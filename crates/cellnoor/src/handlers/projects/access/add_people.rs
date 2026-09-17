use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::Relation;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn add_people_to_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: project_id }): Path<IdParam>,
    Json(people): Json<Vec<Uuid>>,
) -> Result<Json<()>, Error> {
    state
        .in_transaction(user, async |tx| {
            insert_project_accesses(tx, project_id, &people).await
        })
        .await
}

pub(in super::super) async fn insert_project_accesses(
    tx: &db::Transaction<'_>,
    project_id: Uuid,
    members: &[Uuid],
) -> Result<(), ErrorInner> {
    let accesses: Vec<_> = members
        .iter()
        .map(|&principal_id| NewProjectAccess {
            project_id,
            principal_id,
        })
        .collect();

    tx.insert_many(&accesses).await
}

struct NewProjectAccess {
    project_id: Uuid,
    principal_id: Uuid,
}

impl Relation for NewProjectAccess {
    const NAME: &'static str = "project_access";
}

impl Insert for NewProjectAccess {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            project_id,
            principal_id,
        } = self;

        vec![("project_id", project_id), ("principal_id", principal_id)]
    }
}
