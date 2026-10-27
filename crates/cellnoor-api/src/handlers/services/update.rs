use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::service::{Service, ServiceUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{IdParam, services::index::select_service_by_id, set_is_staff},
    state::AppState,
};

pub async fn update_service(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(service): crate::extract::JsonExtractor<ServiceUpdate>,
) -> Result<Json<Service>, DbError> {
    state
        .in_transaction(user, async |tx| {
            update_service_by_id(tx, id, &service).await
        })
        .await
}

pub(in super::super) async fn update_service_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &ServiceUpdate,
) -> Result<Service, DbError> {
    tx.update(id, update).await?;
    set_is_staff(tx, id, update.is_staff).await?;

    select_service_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::service::ServiceUpdate;

    use crate::{
        handlers::services::{create::test::insert_test_service, update::update_service_by_id},
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_service(&tx, |_| ()).await.unwrap();

        let update = ServiceUpdate {
            description: Some("updated".to_nonempty_string()),
            is_staff: false,
        };

        update_service_by_id(&tx, inserted.id, &update)
            .await
            .unwrap();
    }
}
