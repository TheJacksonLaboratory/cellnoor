use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{auth::AuthUser, db, error::Error, handlers::path::IdParam, state::AppState};

pub async fn delete_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let result = delete_project_by_id(&tx, id).await.map(Json);

    tx.commit().await?;

    result
}

pub async fn delete_project_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), Error> {
    let n = tx
        .execute("delete from project where id = $1", &[&id])
        .await?;

    if n == 0 {
        return Err(Error::resource_not_found());
    }

    Ok(())
}
