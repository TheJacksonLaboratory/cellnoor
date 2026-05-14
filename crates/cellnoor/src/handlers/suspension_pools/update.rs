use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension_pool::{
    NewSuspensionPoolRecord, SuspensionPool, SuspensionPoolUpdate,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, ToUpdateClause},
    },
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
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
) -> Result<Json<SuspensionPool>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_suspension_pool_by_id(&tx, id, &record)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_suspension_pool_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    SuspensionPoolUpdate {
        record,
        measurements,
        preparers,
    }: &SuspensionPoolUpdate,
) -> Result<SuspensionPool, ErrorInner> {
    update_suspension_pool_record(tx, id, record).await?;

    let preparer_insertions = async {
        if !preparers.is_empty() {
            insert_suspension_pool_preparers(tx, id, preparers).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_suspension_pool_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_suspension_pool_by_id(tx, id).await
}

async fn update_suspension_pool_record(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: &NewSuspensionPoolRecord,
) -> Result<(), ErrorInner> {
    let fields = record.as_field_value_pairs();

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(
            &format!("update suspension_pool set {update_clause}"),
            &params,
        )
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound.into());
    }

    Ok(())
}
