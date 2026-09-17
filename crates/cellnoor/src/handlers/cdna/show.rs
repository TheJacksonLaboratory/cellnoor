use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    cdna::{CdnaDetailed, CdnaPredicateInner},
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, cdna::index_detailed::select_cdna_detailed},
    state::AppState,
};

pub async fn show_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<CdnaDetailed>, Error> {
    state
        .in_transaction(user, async |tx| select_cdna_by_id(tx, id).await)
        .await
}

pub(super) async fn select_cdna_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<CdnaDetailed, ErrorInner> {
    tx.select_one(
        CdnaPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_cdna_detailed,
    )
    .await
}
