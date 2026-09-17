use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    operator::UuidOperator,
    specimen::{SpecimenDetailed, SpecimenPredicate},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, specimens::index_detailed::select_specimens_detailed},
    state::AppState,
};

pub async fn show_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<SpecimenDetailed>, Error> {
    state
        .in_transaction(user, async |tx| select_specimen_by_id(tx, id).await)
        .await
}

pub(super) async fn select_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SpecimenDetailed, ErrorInner> {
    tx.select_one(
        SpecimenPredicate::Id(UuidOperator::Eq(id)),
        select_specimens_detailed,
    )
    .await
}
