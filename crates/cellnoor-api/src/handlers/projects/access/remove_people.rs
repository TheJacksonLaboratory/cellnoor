use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    handlers::IdParam,
    state::AppState,
};

pub async fn remove_people_from_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id: project_id }): Path<IdParam>,
    crate::extract::JsonExtractor(people): crate::extract::JsonExtractor<Vec<Uuid>>,
) -> Result<Json<()>, DbError> {
    state
        .in_transaction(user, async |tx| {
            delete_project_accesses(tx, project_id, &people).await
        })
        .await
}

pub(in super::super::super) async fn delete_project_accesses(
    tx: &db::Transaction<'_>,
    project_id: Uuid,
    people: &[Uuid],
) -> Result<(), DbError> {
    static DELETE_PROJECT_ACCESSES: &str =
        "delete from project_access where project_id = $1 and principal_id = any($2)";

    let n = tx
        .execute(&Sql::new(DELETE_PROJECT_ACCESSES, vec![&project_id, &people]))
        .await?;

    if n == 0 {
        return Err(DbError::ResourceNotFound);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::{
            projects::{
            access::remove_people::delete_project_accesses, create::insert_test_project,
            show::select_project_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn remove_people() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, project) = insert_test_project(&tx, |_| ()).await.unwrap();
        let id = project.record.project.id;

        delete_project_accesses(&tx, id, &project.record.members)
            .await
            .unwrap();

        let selected = select_project_by_id(&tx, id).await.unwrap();

        assert!(selected.record.members.is_empty());
    }
}
