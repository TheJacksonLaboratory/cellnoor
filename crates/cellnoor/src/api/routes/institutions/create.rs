use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, NewInstitution};

use crate::{
    auth::AuthUser,
    db::{self},
    error,
    state::AppState,
};

pub async fn create_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, error::Error> {
    insert_institution(institution, &mut state.db_client(user).await?)
        .await
        .map(Json)
}

pub async fn insert_institution(
    NewInstitution {
        name,
        microsoft_entra_tenant_id,
    }: NewInstitution,
    db_client: &mut db::Client,
) -> Result<Institution, error::Error> {
    let tx = db_client.begin().await?;
    let institution = tx
        .query_one_scalar(
            include_str!("queries/insert.sql"),
            &[&name, &microsoft_entra_tenant_id],
        )
        .await?;

    Ok(institution)
}
