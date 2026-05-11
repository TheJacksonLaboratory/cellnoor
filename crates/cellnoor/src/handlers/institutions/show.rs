use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    UuidOperator,
    institution::{Institution, InstitutionPredicate},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, util::select_one},
    error::{Error, ErrorInner},
    handlers::{institutions::index::select_institutions, path::IdParam},
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
