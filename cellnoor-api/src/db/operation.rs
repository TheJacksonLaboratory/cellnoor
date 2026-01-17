use diesel::PgConnection;

use crate::{
    api::AuthenticatedUser,
    db::{self, DbConnection},
};

pub trait Operation<Output>: Sized {
    async fn fetch_authorization_data(
        &self,
        _user: &AuthenticatedUser,
        _db_conn: DbConnection,
    ) -> Result<(), db::Error> {
        tracing::info!("user is successfuly authenticated");
        Ok(())
    }

    async fn fetch_validation_data(&self, _db_conn: DbConnection) -> Result<(), db::Error> {
        tracing::info!("user is authenticated");
        Ok(())
    }

    fn authorize(self, _user: &AuthenticatedUser) -> Self {
        self
    }

    fn validate(&self) {}

    fn execute(self, db_conn: &mut PgConnection) -> Result<Output, super::Error>;
}
