use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    person::{PermissionsToGrant, PermissionsToRevoke},
    service::{NewService, Service, ServiceUpdate},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        people::create::{PermissionSet, permission_to_permission_set},
        services::{access::add_people::insert_service_accesses, index::select_service_by_id},
    },
    state::AppState,
};

pub async fn update_service(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(serivce): Json<ServiceUpdate>,
) -> Result<Json<Service>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_service_by_id(&tx, id, &serivce).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub(in super::super) async fn update_service_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    ServiceUpdate {
        record,
        permissions_to_grant,
        permissions_to_revoke,
    }: &ServiceUpdate,
) -> Result<Service, ErrorInner> {
    db::update(tx, "service", id, record).await?;

    if let Some(permissions) = permissions_to_grant {
        grant_permissions(tx, id, permissions).await?;
    }

    if let Some(permissions) = permissions_to_revoke {
        revoke_permissions(tx, id, permissions).await?;
    }

    select_service_by_id(tx, id).await
}

async fn grant_permissions(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions_to_grant: &PermissionsToGrant,
) -> Result<(), ErrorInner> {
    let permissions_to_grant: Vec<_> = permissions_to_grant
        .iter()
        .map(permission_to_permission_set)
        .collect();

    tx.execute_raw_sql(
        "grant_permissions_to_service($1, $2)",
        &[&user_id, &permissions_to_grant],
    )
    .await?;

    Ok(())
}

async fn revoke_permissions(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions_to_revoke: &PermissionsToRevoke,
) -> Result<(), ErrorInner> {
    let permissions_to_revoke: Vec<_> = permissions_to_revoke
        .iter()
        .map(permission_to_permission_set)
        .collect();

    tx.execute_raw_sql(
        "revoke_permissions_from_service($1, $2)",
        &[&user_id, &permissions_to_revoke],
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        id::NoId,
        service::{NewService, NewServiceRecord, ServiceUpdate},
    };

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
            record: NewServiceRecord {
                id: NoId {},
                description: Some("updated".to_nonempty_string()),
                can_admin_all_projects: false,
                can_admin_users: false,
            },
            permissions_to_grant: None,
            permissions_to_revoke: None,
        };

        update_service_by_id(&tx, *inserted.id, &update)
            .await
            .unwrap();
    }
}
