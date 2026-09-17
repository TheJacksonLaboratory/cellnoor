use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    chromium_run::{ChromiumRunDetailed, ChromiumRunPredicateInner},
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, chromium_runs::index_detailed::select_chromium_runs_detailed},
    state::AppState,
};

pub async fn show_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<ChromiumRunDetailed>, Error> {
    state
        .in_transaction(user, async |tx| select_chromium_run_by_id(tx, id).await)
        .await
}

// Visibility required for tests
pub(super) async fn select_chromium_run_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<ChromiumRunDetailed, ErrorInner> {
    tx.select_one(
        ChromiumRunPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_chromium_runs_detailed,
    )
    .await
}
