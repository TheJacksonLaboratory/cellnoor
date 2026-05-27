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
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{IdParam, specimens::index_detailed::select_specimens_detailed},
    state::AppState,
};

pub async fn show_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<SpecimenDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_specimen_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// This visibility is necessary for RLS tests
pub(in crate::handlers) async fn select_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<SpecimenDetailed, ErrorInner> {
    select_one(
        tx,
        SpecimenPredicate::Id(UuidOperator::Eq(id)),
        select_specimens_detailed,
    )
    .await
}
