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
        util::{AsFieldValuePairs, ToUpdateClause},
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
    updated_record: &NewInstitution,
) -> Result<Institution, ErrorInner> {
    let fields = updated_record.as_field_value_pairs();
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
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        institution::{Institution, NewInstitution, SavedInstitutionRecord},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
        handlers::institutions::{
            create::test::insert_test_institution, update::update_institution_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            mut pre_update,
            Institution {
                record: SavedInstitutionRecord { id, .. },
                links: _,
            },
        ) = insert_test_institution(&tx, identity).await;
        pre_update.name = "updated".to_nonempty_string();

        let Institution {
            record: post_update_record,
            links: _,
        } = update_institution_by_id(&tx, *id, &pre_update)
            .await
            .unwrap();

        let expected_record = SavedInstitutionRecord {
            id,
            name: pre_update.name,
            microsoft_entra_tenant_id: pre_update.microsoft_entra_tenant_id,
        };

        assert_eq!(post_update_record, expected_record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = update_institution_by_id(
            &tx,
            Uuid::new_v4(),
            &NewInstitution {
                id: NoId {},
                name: "updated".to_nonempty_string(),
                microsoft_entra_tenant_id: Uuid::new_v4(),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
