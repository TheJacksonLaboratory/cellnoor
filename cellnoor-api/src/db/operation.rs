use diesel::PgConnection;
use uuid::Uuid;

use crate::{
    api::{
        self, AuthenticatedUser,
        auth::{self, AuthorizationData},
    },
    db::{self, DbConnection},
    state::AppState,
};

pub trait Operation<Output>: Sized {
    type Authorized;
    type ValidationData;

    async fn fetch_validation_data(
        &self,
        db_conn: DbConnection,
    ) -> Result<Self::ValidationData, db::Error>;

    fn authorize(
        self,
        authorization_data: AuthorizationData,
    ) -> Result<Self::Authorized, auth::Error>;

    fn validate(
        authorized_request: &Self::Authorized,
        // We require ownership because a lot of the time, `ValidationData = ()`
        validation_data: Self::ValidationData,
    ) -> Result<(), api::DataError>;

    fn execute(
        authorized_request: Self::Authorized,
        db_conn: &mut PgConnection,
    ) -> Result<Output, api::Error>;

    #[cfg(any(feature = "dummy-data", test))]
    fn execute_without_authorization(self, db_conn: &mut PgConnection) -> Output {
        let authorized = self.authorize(AuthorizationData::new_admin()).unwrap();
        Self::execute(authorized, db_conn).unwrap()
    }
}
