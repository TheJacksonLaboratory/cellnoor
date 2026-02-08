use axum::{Extension, Json, extract::State};
use cellnoor_models::suspension::{NewSuspension, Suspension, SuspensionContent};
use cellnoor_schema::{specimens, suspension_preparers, suspensions};
use diesel::{pg::Pg, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::{
        auth::AuthUser, routes::suspensions::show::select_suspension_by_id,
        util::validate_timestamps,
    },
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_suspension(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Json(suspension): Json<NewSuspension>,
) -> Result<Json<Suspension>, db::Error> {
    let SpecimenInfo {
        received_at,
        returned_at,
        project_id,
    } = specimen_info(suspension.parent_specimen_id(), &mut db_conn).await?;

    validate_suspension_created_between_specimen_receipt_and_return(
        suspension.created_at(),
        received_at,
        returned_at,
    )?;

    let suspension_preparers = suspension.preparer_ids().to_vec();

    let suspension_id = db_conn
        .transaction(move |db_conn| {
            insert_suspension_and_preparers(project_id, suspension, suspension_preparers, db_conn)
                .scope_boxed()
        })
        .await?;

    select_suspension_by_id(user.projects(), suspension_id, &mut db_conn)
        .await
        .map(Json)
}

pub async fn insert_suspension_and_preparers(
    project_id: Uuid,
    suspension: NewSuspension,
    preparer_ids: Vec<Uuid>,
    db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let suspension_id = insert_suspension(project_id, suspension, db_conn).await?;

    insert_suspension_preparers(suspension_id, &preparer_ids, db_conn).await?;

    Ok(suspension_id)
}

pub async fn insert_suspension(
    project_id: Uuid,
    suspension: NewSuspension,
    mut db_conn: &diesel_async::AsyncPgConnection,
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
        .get_result(&mut db_conn)
        .await?)
}

pub(super) async fn insert_suspension_preparers(
    suspension_id: Uuid,
    preparer_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
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
        .execute(&mut db_conn)
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

#[derive(HasQuery)]
#[diesel(check_for_backend(Pg), table_name=specimens)]
pub struct SpecimenInfo {
    #[diesel(deserialize_as=jiff_diesel::Timestamp)]
    pub received_at: Timestamp,
    #[diesel(deserialize_as=jiff_diesel::NullableTimestamp)]
    pub returned_at: Option<Timestamp>,
    project_id: Uuid,
}

async fn specimen_info(
    specimen_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<SpecimenInfo, db::Error> {
    Ok(SpecimenInfo::query()
        .find(specimen_id)
        .first(&mut db_conn)
        .await?)
}
