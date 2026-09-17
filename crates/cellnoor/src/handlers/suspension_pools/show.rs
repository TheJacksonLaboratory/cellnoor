use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    operator::UuidOperator,
    suspension_pool::{SuspensionPoolDetailed, SuspensionPoolPredicateInner},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{IdParam, suspension_pools::index_detailed::select_suspension_pools_detailed},
    state::AppState,
};

pub async fn show_suspension_pool(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<SuspensionPoolDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| select_suspension_pool_by_id(tx, id).await)
        .await
}

pub(super) async fn select_suspension_pool_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SuspensionPoolDetailed, DbError> {
    tx.select_one(
        SuspensionPoolPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_suspension_pools_detailed,
    )
    .await
}
