use diesel::prelude::*;
use scamplers_models::{
    suspension::{
        Suspension, SuspensionContent, SuspensionCreation, SuspensionCreationInner,
        SuspensionFields, SuspensionId,
    },
    suspension_pool::SuspensionPoolId,
};
use scamplers_schema::{suspension_preparers, suspension_tagging, suspensions};
use uuid::Uuid;

use crate::db;

impl db::Operation<Suspension> for (SuspensionCreation, SuspensionContent) {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Suspension, db::Error> {
        let (suspension_creation, content) = self;

        let SuspensionCreation(SuspensionCreationInner {
            inner,
            preparer_ids,
            tag_ids,
        }) = suspension_creation;

        let suspension_id = insert_suspension(inner, content, db_conn)?;
        insert_suspension_preparers(suspension_id, preparer_ids.as_ref(), db_conn)?;
        if let Some(tag_ids) = tag_ids {
            insert_suspension_tags(suspension_id, None, &tag_ids, db_conn)?;
        }

        suspension_id.execute(db_conn)
    }
}

pub fn insert_suspension(
    fields: SuspensionFields,
    content: SuspensionContent,
    db_conn: &mut PgConnection,
) -> Result<SuspensionId, db::Error> {
    Ok(diesel::insert_into(suspensions::table)
        .values((fields, suspensions::content.eq(content)))
        .returning(suspensions::id)
        .get_result(db_conn)?)
}

pub fn insert_suspension_preparers(
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

pub fn insert_suspension_tags(
    suspension_id: SuspensionId,
    pool_id: Option<SuspensionPoolId>,
    tag_ids: &[Uuid],
    db_conn: &mut PgConnection,
) -> Result<(), db::Error> {
    let tag_mappings: Vec<_> = tag_ids
        .iter()
        .map(|p| {
            (
                suspension_tagging::suspension_id.eq(suspension_id),
                pool_id.map(|i| suspension_tagging::pool_id.eq(i)),
                suspension_tagging::tag_id.eq(p),
            )
        })
        .collect();

    diesel::insert_into(suspension_tagging::table)
        .values(tag_mappings)
        .execute(db_conn)?;

    Ok(())
}
