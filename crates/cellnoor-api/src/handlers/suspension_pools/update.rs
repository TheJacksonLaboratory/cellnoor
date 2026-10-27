use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension_pool::{SuspensionPoolDetailed, SuspensionPoolUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{
        IdParam,
        suspension_pools::{
            create::insert_suspension_pool_preparers,
            measurements::create::insert_suspension_pool_measurements,
            show::select_suspension_pool_by_id,
        },
    },
    state::AppState,
};

pub async fn update_suspension_pool(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(record): crate::extract::JsonExtractor<SuspensionPoolUpdate>,
) -> Result<Json<SuspensionPoolDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| {
            update_suspension_pool_by_id(tx, id, &record).await
        })
        .await
}

async fn update_suspension_pool_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: &SuspensionPoolUpdate,
) -> Result<SuspensionPoolDetailed, DbError> {
    tx.update(id, update).await?;

    select_suspension_pool_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::suspension_pool::{NewSuspensionPoolRecord, SuspensionPoolUpdate};

    use crate::{
        handlers::suspension_pools::{
            create::insert_test_suspension_pool_and_suspensions,
            update::update_suspension_pool_by_id,
        },
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
            .await
            .unwrap();
        let id = *inserted.record.id;

        let pre_update = SuspensionPoolUpdate {
            readable_id: crate::db::dummy_data::random_name_for("suspension_pool"),
            name: "updated".to_nonempty_string(),
            ..insert_input.record
        };

        update_suspension_pool_by_id(&tx, id, &pre_update)
            .await
            .unwrap();
    }
}
