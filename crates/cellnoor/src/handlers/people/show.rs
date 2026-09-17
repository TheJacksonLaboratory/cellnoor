use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    operator::UuidOperator,
    person::{Person, PersonPredicate},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{IdParam, people::index::select_people},
    state::AppState,
};

pub async fn show_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<Person>, Error> {
    state
        .in_transaction(user, async |tx| select_person_by_id(tx, id).await)
        .await
}

pub(super) async fn select_person_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Person, ErrorInner> {
    tx.select_one(PersonPredicate::Id(UuidOperator::Eq(id)), select_people)
        .await
}
