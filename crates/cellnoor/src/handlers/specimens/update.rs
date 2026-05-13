use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::{
    Specimen,
    creation::{NewSpecimen, NewSpecimenRecord},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, FieldValuePairs, ToUpdateClause},
    },
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        specimens::{
            measurements::create::insert_specimen_measurement, show::select_specimen_by_id,
        },
    },
    state::AppState,
};

pub async fn update_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<Specimen>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_specimen_by_id(&tx, id, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: NewSpecimen,
) -> Result<Specimen, ErrorInner> {
    let (record, measurements) = record.split_for_insertion();

    update_specimen_record(tx, id, &record).await?;

    futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_specimen_measurement(tx, id, m)),
    )
    .await?;

    select_specimen_by_id(tx, id).await
}

async fn update_specimen_record(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: &NewSpecimenRecord,
) -> Result<(), ErrorInner> {
    let fields = record.as_field_value_pairs();

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update specimen set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound.into());
    }

    Ok(())
}
