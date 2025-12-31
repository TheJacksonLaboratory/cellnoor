use cellnoor_models::institution::InstitutionCreation;
use cellnoor_schema::institutions::dsl::{id, institutions};
use diesel::{PgConnection, prelude::*};

use crate::initial_data::Upsert;

impl Upsert for InstitutionCreation {
    fn upsert(self, db_conn: &mut PgConnection) -> anyhow::Result<()> {
        diesel::insert_into(institutions)
            .values(&self)
            .on_conflict(id)
            .do_update()
            .set(&self)
            .execute(db_conn)?;

        Ok(())
    }
}
