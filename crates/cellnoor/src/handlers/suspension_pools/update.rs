use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension_pool::{SuspensionPoolDetailed, SuspensionPoolUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        IdParam,
        suspension_pools::{
            create::insert_suspension_pool_preparers,
            measurements::create::insert_suspension_pool_measurement,
            show::select_suspension_pool_by_id,
        },
    },
    state::AppState,
};

pub async fn update_suspension_pool(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<SuspensionPoolUpdate>,
) -> Result<Json<SuspensionPoolDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_suspension_pool_by_id(&tx, id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_suspension_pool_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    SuspensionPoolUpdate {
        record,
        measurements,
        preparers,
    }: &SuspensionPoolUpdate,
) -> Result<SuspensionPoolDetailed, ErrorInner> {
    db::update(tx, "suspension_pool", id, record).await?;

    let preparer_insertions = async {
        if let Some(preparers) = preparers {
            insert_suspension_pool_preparers(tx, id, preparers).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .flatten()
            .map(|m| insert_suspension_pool_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_suspension_pool_by_id(tx, id).await
}
