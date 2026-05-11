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
    error::{Error, ErrorInner},
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
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_institution_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    NewInstitution {
        name,
        microsoft_entra_tenant_id,
    }: &NewInstitution,
) -> Result<Institution, ErrorInner> {
    let fields: FieldValuePairs<_> = [
        ("name", name),
        ("microsoft_entra_tenant_id", microsoft_entra_tenant_id),
    ];

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update institution set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    select_institution_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::institution::{InstitutionRecord, NewInstitution};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::{Error, ErrorInner},
        handlers::institutions::update::update_institution_by_id,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let name = "Not The Jackson Laboratory".to_nonempty_string();
        let updated_institution = update_institution_by_id(
            &tx,
            Uuid::nil(),
            &NewInstitution {
                name: name.clone(),
                microsoft_entra_tenant_id: Uuid::max(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            updated_institution.record,
            InstitutionRecord {
                id: Uuid::nil(),
                name,
                microsoft_entra_tenant_id: Uuid::max()
            }
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = update_institution_by_id(
            &tx,
            Uuid::new_v4(),
            &NewInstitution {
                name: "foo".to_nonempty_string(),
                microsoft_entra_tenant_id: Uuid::new_v4(),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
