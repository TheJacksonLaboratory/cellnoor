use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::service::{Service, ServiceUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        permissions::{grant_permissions, revoke_permissions},
        services::index::select_service_by_id,
        set_is_staff,
    },
    state::AppState,
};

pub async fn update_service(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(service): Json<ServiceUpdate>,
) -> Result<Json<Service>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_service_by_id(&tx, id, &service).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn update_service_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ServiceUpdate,
) -> Result<Service, ErrorInner> {
    let ServiceUpdate {
        record,
        permissions_to_grant,
        permissions_to_revoke,
    } = update;

    tx.update(id, record).await?;
    set_is_staff(tx, id, record.is_staff).await?;
    grant_permissions(tx, id, permissions_to_grant).await?;
    revoke_permissions(tx, id, permissions_to_revoke).await?;

    select_service_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::service::{ServiceSimpleFields, ServiceUpdate};

    use crate::{
        handlers::services::{create::test::insert_test_service, update::update_service_by_id},
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_service(&tx, |_| ()).await.unwrap();

        let update = ServiceUpdate {
            record: ServiceSimpleFields {
                description: Some("updated".to_nonempty_string()),
                is_staff: false,
            },
            permissions_to_grant: Vec::new(),
            permissions_to_revoke: Vec::new(),
        };

        update_service_by_id(&tx, inserted.id, &update)
            .await
            .unwrap();
    }
}
