use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::chromium_run::{ChromiumRun, ChromiumRunUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{chromium_runs::show::select_chromium_run_by_id, path::IdParam},
    state::AppState,
};

pub async fn update_chromium_run(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<ChromiumRunUpdate>,
) -> Result<Json<ChromiumRun>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_chromium_run_by_id(&tx, id, &record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_chromium_run_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ChromiumRunUpdate,
) -> Result<ChromiumRun, ErrorInner> {
    db::update(tx, "chromium_run", id, update).await?;
    select_chromium_run_by_id(tx, id).await
}
