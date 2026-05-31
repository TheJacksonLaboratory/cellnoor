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
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{IdParam, libraries::index_detailed::select_libraries_detailed},
    state::AppState,
};

pub async fn show_library(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<LibraryDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_library_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(in super::super) async fn select_library_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<LibraryDetailed, ErrorInner> {
    select_one(
        tx,
        LibraryPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_libraries_detailed,
    )
    .await
}
