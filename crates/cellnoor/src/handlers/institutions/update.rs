use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::institution::{Institution, NewInstitution};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToUpdateClause},
    },
    error::Error,
    handlers::{institutions::show::select_institution_by_id, path::IdParam},
    state::AppState,
};

pub async fn update_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_institution_by_id(&tx, id, &institution)
        .await
        .map(Json);

    tx.commit().await?;

    response
}

pub async fn update_institution_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    NewInstitution {
        name,
        microsoft_entra_tenant_id,
    }: &NewInstitution,
) -> Result<Institution, Error> {
    let fields: FieldValuePairs<_> = [
        ("name", name),
        ("microsoft_entra_tenant_id", microsoft_entra_tenant_id),
    ];

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update institution set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(Error::resource_not_found());
    }

    select_institution_by_id(tx, id).await
}
