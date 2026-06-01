use axum::{Json, extract::State};
use cellnoor_types::service_account::{NewServiceAccount, ServiceAccount, ServiceAccountField};

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::service_accounts::{
        access::add_people::insert_service_account_accesses, index::select_service_account_by_id,
    },
    state::AppState,
};

pub async fn create_service_account(
    State(state): State<AppState>,
    user: AuthUser,
    Json(service_account): Json<NewServiceAccount>,
) -> Result<Json<ServiceAccount>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_service_account(&tx, &service_account)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_service_account(
    tx: &db::Transaction<'_>,
    new_record: &NewServiceAccount,
) -> Result<ServiceAccount, ErrorInner> {
    let id = db::insert_into(tx, "service_account", new_record).await?;

    insert_service_account_accesses(tx, id, &new_record.users).await?;

    select_service_account_by_id(tx, id).await
}

// `owned_by` is intentionally omitted: the database fills it from
// `current_user::uuid`, and RLS guarantees it equals the caller. The only
// column a user writes on create is `description`.
impl AsFieldValuePairs<ServiceAccountField, 1> for NewServiceAccount {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, ServiceAccountField, 1> {
        use ServiceAccountField::*;

        let Self {
            description,
            users: _,
        } = self;

        [(Description, description)]
    }
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::service_account::{NewServiceAccount, ServiceAccount};
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::service_accounts::create::insert_service_account,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_service_account<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewServiceAccount, ServiceAccount), ErrorInner>
    where
        F: FnMut(&mut NewServiceAccount),
    {
        let mut new = NewServiceAccount {
            description: Some(Uuid::new_v4().to_string().to_nonempty_string()),
            users: vec![Uuid::nil()],
        };

        modify(&mut new);

        let inserted = insert_service_account(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_service_account(&tx, |_| ()).await.unwrap();
    }
}
