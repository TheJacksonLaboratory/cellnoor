use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    operator::UuidOperator,
    suspension::{SuspensionDetailed, SuspensionPredicateInner},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, suspensions::index_detailed::select_suspensions_detailed},
    state::AppState,
};

pub async fn show_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<SuspensionDetailed>, Error> {
    state
        .in_transaction(user, async |tx| select_suspension_by_id(tx, id).await)
        .await
}

pub(super) async fn select_suspension_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SuspensionDetailed, ErrorInner> {
    tx.select_one(
        SuspensionPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_suspensions_detailed,
    )
    .await
}
