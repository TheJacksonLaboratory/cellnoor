use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    library::{LibraryDetailed, LibraryPredicateInner},
    operator::UuidOperator,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{IdParam, libraries::index_detailed::select_libraries_detailed},
    state::AppState,
};

pub async fn show_library(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<LibraryDetailed>, Error> {
    state
        .in_transaction(user, async |tx| select_library_by_id(tx, id).await)
        .await
}

pub(super) async fn select_library_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<LibraryDetailed, ErrorInner> {
    tx.select_one(
        LibraryPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_libraries_detailed,
    )
    .await
}
