use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, InstitutionField, NewInstitution};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::institutions::show::select_institution_by_id,
    state::AppState,
};

pub async fn create_institution(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(institution): crate::extract::JsonExtractor<NewInstitution>,
) -> Result<Json<Institution>, DbError> {
    state
        .in_transaction(user, async |tx| insert_institution(tx, &institution).await)
        .await
}

async fn insert_institution(
    tx: &db::Transaction<'_>,
    new_record: &NewInstitution,
) -> Result<Institution, DbError> {
    let id = tx.insert_returning_id(new_record).await?;

    select_institution_by_id(tx, id).await
}

impl Insert for NewInstitution {
    type Field = InstitutionField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use InstitutionField::*;
        let Self {
            id: _,
            name,
            microsoft_entra_tenant_id,
        } = self;

        vec![
            (Name, name),
            (MicrosoftEntraTenantId, microsoft_entra_tenant_id),
        ]
    }
}

#[cfg(any(test, feature = "dev"))]
pub async fn insert_test_institution<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewInstitution, Institution), DbError>
where
    F: FnMut(&mut NewInstitution),
{
    use cellnoor_types::id::NoId;
    use uuid::Uuid;

    use crate::state::dev_util::ToNonemptyString;

    let mut new = NewInstitution {
        id: NoId,
        name: Uuid::new_v4().to_string().to_nonempty_string(),
        microsoft_entra_tenant_id: Uuid::new_v4(),
    };

    modify(&mut new);

    let inserted = insert_institution(tx, &new).await?;
    Ok((new, inserted))
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        id::NoId,
        institution::{Institution, NewInstitution},
    };
    use uuid::Uuid;

    use super::insert_test_institution;
    use crate::{
        db::{self, DbError},
        handlers::institutions::create::insert_institution,
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_institution(&tx, |_| ()).await.unwrap();
    }
}
