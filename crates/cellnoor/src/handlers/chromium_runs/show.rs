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
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{chromium_runs::index_detailed::select_chromium_runs_detailed, path::IdParam},
    state::AppState,
};

pub async fn show_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<ChromiumRunDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_run_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(super) async fn select_chromium_run_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<ChromiumRunDetailed, ErrorInner> {
    select_one(
        tx,
        ChromiumRunPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_chromium_runs_detailed,
    )
    .await
}
