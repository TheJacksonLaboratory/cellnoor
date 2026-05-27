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

pub async fn delete_suspension_pool(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = delete_suspension_pool_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn delete_suspension_pool_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<(), ErrorInner> {
    db::delete_by_id(tx, "suspension_pool", id).await
}
