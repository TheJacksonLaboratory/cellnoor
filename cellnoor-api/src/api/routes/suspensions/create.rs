use crate::state::AppState;
use crate::{
    api::util::validate_timestamps,
    db::{self, DbConnection},
};
use axum::{Json, extract::State};
use cellnoor_models::suspension::{NewSuspension, Suspension, SuspensionContent};
use cellnoor_schema::suspensions;
use cellnoor_schema::{specimens, suspension_preparers};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

pub(super) async fn create_suspension(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(suspension): Json<NewSuspension>,
) -> Result<Json<Suspension>, db::Error> {
    let (specimen_received_at, specimen_returned_at, project_id) =
        specimen_info(suspension.parent_specimen_id(), &mut db_conn).await?;

    validate_suspension_created_between_specimen_receipt_and_return(
        suspension.created_at(),
        specimen_received_at,
        specimen_returned_at,
    )?;

    let suspension_preparers = suspension.preparer_ids().to_vec();
    let suspension_id = insert_suspension(project_id, suspension, &mut db_conn).await?;

    insert_suspension_preparers(suspension_id, &suspension_preparers, &mut db_conn).await?;

    todo!()
}

pub(super) async fn insert_suspension(
    project_id: Uuid,
    suspension: NewSuspension,
    db_conn: &mut DbConnection,
) -> Result<Uuid, db::Error> {
    let (suspension, suspension_content, lysis_duration_minutes) = match suspension {
        NewSuspension::Cell(s) => (s, SuspensionContent::Cells, None),
        NewSuspension::Nucleus {
            common,
            lysis_duration_minutes,
        } => (
            common,
            SuspensionContent::Nuclei,
            Some(lysis_duration_minutes),
        ),
    };

    Ok(diesel::insert_into(suspensions::table)
        .values((
            suspensions::project_id.eq(project_id),
            suspension,
            suspensions::content.eq(suspension_content),
            suspensions::lysis_duration_minutes.eq(lysis_duration_minutes),
        ))
        .returning(suspensions::id)
        .get_result(db_conn)
        .await?)
}

pub(super) async fn insert_suspension_preparers(
    suspension_id: Uuid,
    preparer_ids: &[Uuid],
    db_conn: &mut DbConnection,
) -> Result<(), db::Error> {
    let preparer_mappings: Vec<_> = preparer_ids
        .iter()
        .map(|p| {
            (
                suspension_preparers::suspension_id.eq(suspension_id),
                suspension_preparers::prepared_by.eq(p),
            )
        })
        .collect();

    diesel::insert_into(suspension_preparers::table)
        .values(preparer_mappings)
        .execute(db_conn)
        .await?;

    Ok(())
}

pub(super) fn validate_suspension_created_between_specimen_receipt_and_return(
    suspension_created_at: Option<Timestamp>,
    specimen_received_at: Timestamp,
    specimen_returned_at: Option<Timestamp>,
) -> Result<(), db::Error> {
    let Some(created_at) = suspension_created_at else {
        return Ok(());
    };

    validate_timestamps(
        (specimen_received_at, "specimen_received_at"),
        (created_at, "suspension_created_at"),
    )?;

    let Some(specimen_returned_at) = specimen_returned_at else {
        return Ok(());
    };

    validate_timestamps(
        (created_at, "suspension_created_at"),
        (specimen_returned_at, "specimen_returned_at"),
    )?;

    Ok(())
}

pub(super) async fn specimen_info(
    specimen_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<(Timestamp, Option<jiff::Timestamp>, Uuid), db::Error> {
    Ok(specimens::table
        .select((
            specimens::received_at,
            specimens::returned_at,
            specimens::project_id,
        ))
        .find(specimen_id)
        .first(db_conn)
        .await
        .map(
            |(t1, t2, project_id): (jiff_diesel::Timestamp, jiff_diesel::NullableTimestamp, _)| {
                (t1.to_jiff(), t2.to_jiff(), project_id)
            },
        )?)
}
