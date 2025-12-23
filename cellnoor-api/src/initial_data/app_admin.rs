use cellnoor_models::person::{PersonCreation, UserRole};
use cellnoor_schema::people::dsl::{email, id, microsoft_entra_oid, people};
use diesel::{
    PgConnection,
    prelude::*,
    sql_types::{Array, Text},
};
use uuid::Uuid;

use crate::initial_data::Upsert;

impl Upsert for PersonCreation {
    fn upsert(mut self, db_conn: &mut PgConnection) -> anyhow::Result<()> {
        define_sql_function! {fn create_user_if_not_exists(user_id: Text, password: Text, roles: Array<Text>)}

        diesel::update(people)
            .filter(email.eq(self.email()))
            .set(email.eq(None::<String>))
            .execute(db_conn)?;

        let created_user_id: Uuid = diesel::insert_into(people)
            .values(&self)
            .on_conflict(microsoft_entra_oid)
            .do_update()
            .set(&self)
            .returning(id)
            .get_result(db_conn)?;

        // Create a db user corresponding to this person so we can assign them a role.
        // Note that we set a random password so that nobody can log into the database
        // as that user.
        self.roles_mut().push(UserRole::AppAdmin);

        diesel::select(create_user_if_not_exists(
            created_user_id.to_string(),
            Uuid::now_v7().to_string(),
            self.roles(),
        ))
        .execute(db_conn)?;

        Ok(())
    }
}
