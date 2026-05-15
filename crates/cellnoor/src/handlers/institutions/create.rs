use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, InstitutionField, NewInstitution};

use crate::{
    auth::AuthUser,
    db::{self, Record, ToRecord},
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

pub async fn insert_institution(
    tx: &db::Transaction<'_>,
    new_record: &NewInstitution,
) -> Result<Institution, ErrorInner> {
    let id = db::insert_into(tx, "institution", &[new_record]).await?;

    select_institution_by_id(tx, id).await
}

impl ToRecord<InstitutionField, 2> for NewInstitution {
    fn to_record(&self) -> Record<InstitutionField, 2> {
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
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        institution::{Institution, NewInstitution},
    };
    use uuid::Uuid;

    use crate::{
        db,
        handlers::institutions::create::insert_institution,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_institution<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> (NewInstitution, Institution)
    where
        F: FnMut(&mut NewInstitution),
    {
        let mut new = NewInstitution {
            id: NoId {},
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            microsoft_entra_tenant_id: Uuid::new_v4(),
        };

        modify(&mut new);

        let inserted = insert_institution(tx, &new).await.unwrap();
        (new, inserted)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_institution(&tx, identity).await;
    }
}
