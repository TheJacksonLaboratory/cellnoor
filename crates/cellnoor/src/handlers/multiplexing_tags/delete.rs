use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::multiplexing_tag::MultiplexingTag;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::IdParam,
    state::AppState,
};

pub async fn delete_multiplexing_tag(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = delete_multiplexing_tag_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn delete_multiplexing_tag_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<(), ErrorInner> {
    tx.delete::<MultiplexingTag>(id).await
}
