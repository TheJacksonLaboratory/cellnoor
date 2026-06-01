use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::service_account::{NewServiceAccount, ServiceAccount};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{IdParam, service_accounts::index::select_service_account_by_id},
    state::AppState,
};

pub async fn update_service_account(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(service_account): Json<NewServiceAccount>,
) -> Result<Json<ServiceAccount>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_service_account_by_id(&tx, id, &service_account)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn update_service_account_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    updated_record: &NewServiceAccount,
) -> Result<ServiceAccount, ErrorInner> {
    db::update(tx, "service_account", id, updated_record).await?;

    select_service_account_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::service_account::NewServiceAccount;

    use crate::{
        handlers::service_accounts::{
            create::test::insert_test_service_account, update::update_service_account_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_service_account(&tx, |_| ()).await.unwrap();

        let update = NewServiceAccount {
            description: Some("updated".to_nonempty_string()),
        };

        update_service_account_by_id(&tx, inserted.id, &update)
            .await
            .unwrap();
    }
}
