use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    institution::{Institution, InstitutionPredicate},
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, institutions::index::select_institutions},
    state::AppState,
};

pub async fn show_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<Institution>, Error> {
    state
        .in_transaction(user, async |tx| select_institution_by_id(tx, id).await)
        .await
}

pub(super) async fn select_institution_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Institution, ErrorInner> {
    tx.select_one(
        InstitutionPredicate::Id(UuidOperator::Eq(id)),
        select_institutions,
    )
    .await
}
