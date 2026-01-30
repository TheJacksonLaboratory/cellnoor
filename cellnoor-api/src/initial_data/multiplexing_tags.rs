use cellnoor_models::multiplexing_tag::MultiplexingTagCreation;
use cellnoor_schema::multiplexing_tags::dsl::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::initial_data::Upsert;

impl Upsert for MultiplexingTagCreation {
    async fn upsert(self, mut db_conn: &AsyncPgConnection) -> anyhow::Result<()> {
        diesel::insert_into(multiplexing_tags)
            .values(&self)
            .on_conflict((tag_id, type_))
            .do_update()
            .set(&self)
            .execute(&mut db_conn)
            .await?;

        Ok(())
    }
}
