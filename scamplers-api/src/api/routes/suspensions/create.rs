use diesel::prelude::*;
use scamplers_models::suspension::SuspensionId;
use scamplers_schema::suspension_preparers;

pub(super) fn insert_suspension_preparers(suspension_id: SuspensionId) {
    let preparer_mappings: Vec<_> = preparer_ids
        .into_iter()
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
}
