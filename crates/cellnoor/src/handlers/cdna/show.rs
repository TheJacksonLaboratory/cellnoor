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
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{IdParam, cdna::index_detailed::select_cdna_detailed},
    state::AppState,
};

pub async fn show_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<CdnaDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_cdna_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in crate::handlers) async fn select_cdna_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<CdnaDetailed, ErrorInner> {
    select_one(
        tx,
        CdnaPredicateInner::Id(UuidOperator::Eq(id)).into(),
        select_cdna_detailed,
    )
    .await
}
