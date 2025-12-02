use diesel::prelude::*;
use scamplers_models::{
    suspension::{SuspensionContent, SuspensionCreationInner},
    suspension_pool::{
        SuspensionPool, SuspensionPoolCreation, SuspensionPoolFields, SuspensionPoolId,
    },
};
use scamplers_schema::{suspension_pool_preparers, suspension_pools};
use uuid::Uuid;

use crate::{
    api::routes::suspensions::{
        insert_suspension, insert_suspension_preparers, insert_suspension_tags,
    },
    db,
};

impl db::Operation<SuspensionPool> for (SuspensionPoolCreation, SuspensionContent) {
    fn execute(self, db_conn: &mut PgConnection) -> Result<SuspensionPool, db::Error> {
        let (
            SuspensionPoolCreation {
                inner,
                preparer_ids,
                suspensions,
            },
            content,
        ) = self;

        let suspension_pool = insert_suspension_pool(inner, db_conn)?;
        let suspension_pool_id = suspension_pool.id().into();

        insert_suspension_pool_preparers(suspension_pool_id, preparer_ids.as_ref(), db_conn)?;

        for SuspensionCreationInner {
            inner,
            preparer_ids,
            tag_ids,
        } in suspensions
        {
            let suspension_id = insert_suspension(inner, content, db_conn)?;
            insert_suspension_preparers(suspension_id, preparer_ids.as_ref(), db_conn)?;
            insert_suspension_tags(
                suspension_id,
                Some(suspension_pool_id),
                tag_ids.as_ref(),
                db_conn,
            )?;
        }

        Ok(suspension_pool)
    }
}

pub(super) fn insert_suspension_pool(
    fields: SuspensionPoolFields,
    db_conn: &mut PgConnection,
) -> Result<SuspensionPool, db::Error> {
    Ok(diesel::insert_into(suspension_pools::table)
        .values(fields)
        .returning(SuspensionPool::as_returning())
        .get_result(db_conn)?)
}

pub(super) fn insert_suspension_pool_preparers(
    pool_id: SuspensionPoolId,
    preparer_ids: &[Uuid],
    db_conn: &mut PgConnection,
) -> Result<(), db::Error> {
    let preparer_mappings: Vec<_> = preparer_ids
        .iter()
        .map(|p| {
            (
                suspension_pool_preparers::pool_id.eq(pool_id),
                suspension_pool_preparers::prepared_by.eq(p),
            )
        })
        .collect();

    diesel::insert_into(suspension_pool_preparers::table)
        .values(preparer_mappings)
        .execute(db_conn)?;

    Ok(())
}
