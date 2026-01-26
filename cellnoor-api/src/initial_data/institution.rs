use cellnoor_models::institution::NewInstitution;
use cellnoor_schema::institutions::dsl::{id, institutions};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::initial_data::Upsert;

impl Upsert for NewInstitution {
    async fn upsert(self, mut db_conn: &AsyncPgConnection) -> anyhow::Result<()> {
        diesel::insert_into(institutions)
            .values(&self)
            .on_conflict(id)
            .do_update()
            .set(&self)
            .execute(&mut db_conn)
            .await?;

        Ok(())
    }
}
