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
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{IdParam, institutions::index::select_institutions},
    state::AppState,
};

pub async fn show_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<Institution>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_institution_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_institution_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Institution, ErrorInner> {
    select_one(
        tx,
        InstitutionPredicate::Id(UuidOperator::Eq(id)),
        select_institutions,
    )
    .await
}
