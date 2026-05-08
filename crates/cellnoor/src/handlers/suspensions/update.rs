use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::suspension::{Suspension, SuspensionUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, JunctionTable, ToUpdateClause, insert_many_to_many},
    },
    error::Error,
    handlers::{
        path::IdParam,
        suspensions::{
            create::insert_suspension_preparers,
            measurements::create::insert_suspension_measurement, show::select_suspension_by_id,
        },
    },
    state::AppState,
};

pub async fn update_suspension(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<SuspensionUpdate>,
) -> Result<Json<Suspension>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_suspension_by_id(&tx, id, record).await.map(Json);

    tx.commit().await?;

    response
}

pub async fn update_suspension_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: SuspensionUpdate,
) -> Result<Suspension, Error> {
    update_suspension_record(tx, id, &record).await?;

    let preparer_insertions = async {
        if let Some(preparers) = &record.preparers {
            insert_suspension_preparers(tx, id, preparers.as_ref()).await
        } else {
            Ok(())
        }
    };

    let measurement_insertions = futures::future::try_join_all(
        record
            .measurements
            .iter()
            .map(|m| insert_suspension_measurement(tx, id, m)),
    );

    tokio::try_join!(preparer_insertions, measurement_insertions)?;

    select_suspension_by_id(tx, id).await
}

async fn update_suspension_record(
    tx: &db::Transaction<'_>,
    id: Uuid,
    SuspensionUpdate {
        readable_id,
        specimen_id,
        content,
        created_at,
        lysis_duration_minutes,
        target_cell_recovery,
        additional_data,
        measurements: _,
        preparers: _,
    }: &SuspensionUpdate,
) -> Result<(), Error> {
    let fields: FieldValuePairs<_> = [
        ("readable_id", readable_id),
        ("specimen_id", specimen_id),
        ("content", content),
        ("created_at", created_at),
        ("lysis_duration_minutes", lysis_duration_minutes),
        ("target_cell_recovery", target_cell_recovery),
        ("additional_data", additional_data),
    ];

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update suspension set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(Error::resource_not_found());
    }

    Ok(())
}
