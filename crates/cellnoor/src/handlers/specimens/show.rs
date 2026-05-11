use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    UuidOperator,
    specimen::{Specimen, SpecimenPredicate},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, util::select_one},
    error::{Error, ErrorInner},
    handlers::{path::IdParam, specimens::index::select_specimens},
    state::AppState,
};

pub async fn show_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<Specimen>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_specimen_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Specimen, ErrorInner> {
    select_one(
        tx,
        SpecimenPredicate::Id(UuidOperator::Eq(id)),
        select_specimens,
    )
    .await
}
