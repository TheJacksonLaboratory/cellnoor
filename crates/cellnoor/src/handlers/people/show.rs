use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    IdParam, UuidOperator,
    institution::{Institution, InstitutionPredicate},
    person::Person,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, util::select_one},
    error::Error,
    state::AppState,
};

pub async fn show_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<Person>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let result = select_person_by_id(&tx, id).await.map(Json);

    tx.commit().await?;

    result
}

pub(super) async fn select_person_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Person, Error> {
    todo!()
}
