use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::path::IdParam,
    state::AppState,
};

pub async fn delete_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = delete_project_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn delete_project_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    let n = tx
        .execute("delete from project where id = $1", &[&id])
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
        handlers::projects::{
            create::test::insert_test_project, delete::delete_project_by_id,
            show::select_project_by_id,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let inserted = insert_test_project(&tx, identity).await;
        let id = *inserted.record().id;

        delete_project_by_id(&tx, id).await.unwrap();

        let error = select_project_by_id(&tx, id).await.unwrap_err();
        assert_eq!(error, ErrorInner::ResourceNotFound);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = delete_project_by_id(&tx, Uuid::new_v4()).await.unwrap_err();
        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
