use cellnoor_models::suspension::{
    Suspension, SuspensionContent, SuspensionCreation, SuspensionId,
};
use cellnoor_schema::{suspension_preparers, suspensions};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db;

impl db::Operation<Suspension> for (SuspensionCreation, SuspensionContent) {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Suspension, db::Error> {
        let (suspension_creation, content) = self;

        let preparer_ids = suspension_creation.preparer_ids().to_vec();

        let suspension_id: SuspensionId = diesel::insert_into(suspensions::table)
            .values((suspension_creation, suspensions::content.eq(content)))
            .returning(suspensions::id)
            .get_result(db_conn)?;

        insert_suspension_preparers(suspension_id, &preparer_ids, db_conn)?;

        suspension_id.execute(db_conn)
    }
}

fn insert_suspension_preparers(
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
