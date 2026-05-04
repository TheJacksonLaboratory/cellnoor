use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, NewInstitution};

use crate::{auth::AuthUser, db, error::Error, state::AppState};

pub async fn create_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, Error> {
    insert_institution(&mut state.db_client(user).await?, institution)
        .await
        .map(Json)
}

async fn insert_institution(
    db_client: &mut db::Client,
    NewInstitution {
        name,
        microsoft_entra_tenant_id,
    }: NewInstitution,
) -> Result<Institution, crate::error::Error> {
    let tx = db_client.begin().await?;

    // Simple queries can be written inline
    let institution = tx
        .query_one_scalar(
            "insert into institution (name, microsoft_entra_tenant_id) values ($1, $2) returning \
             institution",
            &[&name, &microsoft_entra_tenant_id],
        )
        .await?;

    Ok(institution)
}
