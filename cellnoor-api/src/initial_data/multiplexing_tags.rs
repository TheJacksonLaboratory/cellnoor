use cellnoor_models::multiplexing_tag::MultiplexingTagCreation;
use cellnoor_schema::multiplexing_tags::dsl::*;
use diesel::prelude::*;

use crate::initial_data::Upsert;

impl Upsert for MultiplexingTagCreation {
    fn upsert(self, db_conn: &mut diesel::PgConnection) -> anyhow::Result<()> {
        diesel::insert_into(multiplexing_tags)
            .values(&self)
            .on_conflict((tag_id, type_))
            .do_update()
            .set(&self)
            .execute(db_conn)?;

        Ok(())
    }
}
