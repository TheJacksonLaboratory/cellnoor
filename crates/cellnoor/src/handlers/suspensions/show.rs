use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    UuidOperator,
    suspension::{Suspension, SuspensionPredicateInner},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, util::select_one},
    error::Error,
    handlers::{path::IdParam, suspensions::index::select_suspensions},
    state::AppState,
};

pub async fn show_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<Suspension>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let result = select_suspension_by_id(&tx, id).await.map(Json);

    tx.commit().await?;

    result
}

pub async fn select_suspension_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Suspension, Error> {
    select_one(
        tx,
        SuspensionPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_suspensions,
    )
    .await
}
