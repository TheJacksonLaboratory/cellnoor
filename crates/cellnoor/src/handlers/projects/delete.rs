use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::IdParam,
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

async fn delete_project_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    db::delete_by_id(tx, "project", id).await
}

#[cfg(test)]
mod test {

    use crate::{
        handlers::projects::{create::test::insert_test_project, delete::delete_project_by_id},
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_project(&tx, |_| ()).await.unwrap();
        delete_project_by_id(&tx, inserted.record().id)
            .await
            .unwrap();
    }
}
