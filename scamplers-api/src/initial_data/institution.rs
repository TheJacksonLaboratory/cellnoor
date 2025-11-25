use diesel::{PgConnection, prelude::*};
use scamplers_models::institution::InstitutionCreation;
use scamplers_schema::institutions::dsl::{id, institutions};

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
