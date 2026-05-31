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
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{IdParam, suspension_pools::index_detailed::select_suspension_pools_detailed},
    state::AppState,
};

pub async fn show_suspension_pool(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<SuspensionPoolDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspension_pool_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(super) async fn select_suspension_pool_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SuspensionPoolDetailed, ErrorInner> {
    select_one(
        tx,
        SuspensionPoolPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_suspension_pools_detailed,
    )
    .await
}
