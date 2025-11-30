use diesel::prelude::*;
use scamplers_models::suspension::{SuspensionContent, SuspensionFields, SuspensionId};
use scamplers_schema::{suspension_preparers, suspensions};
use uuid::Uuid;

use crate::db;

pub(super) fn insert_suspension(
    fields: SuspensionFields,
    content: SuspensionContent,
    db_conn: &mut PgConnection,
) -> Result<SuspensionId, db::Error> {
    Ok(diesel::insert_into(suspensions::table)
        .values((fields, suspensions::content.eq(content)))
        .returning(suspensions::id)
        .get_result(db_conn)?)
}

pub(super) fn insert_suspension_preparers(
    suspension_id: SuspensionId,
    preparer_ids: &[Uuid],
    db_conn: &mut PgConnection,
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
        .execute(db_conn)?;

    Ok(())
}
