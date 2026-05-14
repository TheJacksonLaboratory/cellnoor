use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    UuidOperator,
    suspension_pool::{SuspensionPool, SuspensionPoolPredicateInner},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, util::select_one},
    error::{Error, ErrorInner},
    handlers::{path::IdParam, suspension_pools::index::select_suspension_pools},
    state::AppState,
};

pub async fn show_suspension_pool(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<SuspensionPool>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspension_pool_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_suspension_pool_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SuspensionPool, ErrorInner> {
    select_one(
        tx,
        SuspensionPoolPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_suspension_pools,
    )
    .await
}

