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
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{path::IdParam, suspensions::index_detailed::select_suspensions_detailed},
    state::AppState,
};

pub async fn show_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<SuspensionDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspension_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// This visibility is necessary for RLS tests
pub(in crate::handlers) async fn select_suspension_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SuspensionDetailed, ErrorInner> {
    select_one(
        tx,
        SuspensionPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_suspensions_detailed,
    )
    .await
}
