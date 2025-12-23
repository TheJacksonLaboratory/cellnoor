use std::fmt::Display;

use diesel::{Connection, PgConnection, RunQueryDsl};

pub trait Operation<Output>: Sized {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Output, super::Error>;

    fn execute_as_user<UserId>(
        self,
        user_id: UserId,
        db_conn: &mut PgConnection,
    ) -> Result<Output, super::Error>
    where
        UserId: Display,
    {
        db_conn.transaction(|tx| {
            diesel::sql_query(format!(r#"set local role "{user_id}""#)).execute(tx)?;

            self.execute(tx)
        })
    }
}
