use cellnoor_models::person::PersonCreation;
use cellnoor_schema::people::{
    dsl::{email, microsoft_entra_oid, people},
    is_admin,
};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::initial_data::Upsert;

impl Upsert for PersonCreation {
    async fn upsert(self, mut db_conn: &AsyncPgConnection) -> anyhow::Result<()> {
        tokio::try_join!(
            diesel::update(people)
                .filter(email.eq(self.email()))
                .set(email.eq(None::<String>))
                .execute(&mut db_conn),
            diesel::insert_into(people)
                .values((&self, is_admin.eq(true)))
                .on_conflict(microsoft_entra_oid)
                .do_update()
                .set(&self)
                .execute(&mut db_conn)
        )?;

        Ok(())
    }
}
