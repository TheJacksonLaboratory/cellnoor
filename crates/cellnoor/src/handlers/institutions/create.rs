use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, InstitutionField, NewInstitution};

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::institutions::show::select_institution_by_id,
    state::AppState,
};

pub async fn create_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_institution(&tx, &institution).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_institution(
    tx: &db::Transaction<'_>,
    new_record: &NewInstitution,
) -> Result<Institution, ErrorInner> {
    let id = db::insert_into(tx, "institution", new_record).await?;

    select_institution_by_id(tx, id).await
}

impl AsFieldValuePairs<InstitutionField, 2> for NewInstitution {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, InstitutionField, 2> {
        use InstitutionField::*;
        let Self {
            id: _,
            name,
            microsoft_entra_tenant_id,
        } = self;

        [
            (Name, name),
            (MicrosoftEntraTenantId, microsoft_entra_tenant_id),
        ]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        id::NoId,
        institution::{Institution, NewInstitution},
    };
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::institutions::create::insert_institution,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_institution<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewInstitution, Institution), ErrorInner>
    where
        F: FnMut(&mut NewInstitution),
    {
        let mut new = NewInstitution {
            id: NoId {},
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            microsoft_entra_tenant_id: Uuid::new_v4(),
        };

        modify(&mut new);

        let inserted = insert_institution(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_institution(&tx, |_| ()).await.unwrap();
    }
}
