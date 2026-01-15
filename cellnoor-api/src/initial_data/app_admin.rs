use cellnoor_models::person::PersonCreation;
use cellnoor_schema::people::{
    dsl::{email, microsoft_entra_oid, people},
    is_admin,
};
use diesel::{PgConnection, prelude::*};

use crate::initial_data::Upsert;

impl Upsert for PersonCreation {
    fn upsert(self, db_conn: &mut PgConnection) -> anyhow::Result<()> {
        diesel::update(people)
            .filter(email.eq(self.email()))
            .set(email.eq(None::<String>))
            .execute(db_conn)?;

        diesel::insert_into(people)
            .values((&self, is_admin.eq(true)))
            .on_conflict(microsoft_entra_oid)
            .do_update()
            .set(&self)
            .execute(db_conn)?;

        Ok(())
    }
}
