use cellnoor_schema::{people, project_people};
use deadpool_diesel::postgres::Pool;
use diesel::{HasQuery, PgConnection, connection::DefaultLoadingMode, prelude::*};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{self, DbConnection};

use super::common::*;

pub type User = StandardClaims;

impl User {
    pub(super) async fn fetch_authorization_data(
        &self,
        db_conn: DbConnection,
    ) -> Result<AuthorizationData, db::Error> {
        let user_id = self.id();

        let user_fields = db_conn.interact(move |db_conn| {
            UserFields::query()
                .filter(people::id.eq(user_id))
                .first(db_conn)
        });

        let user_projects = db_conn.interact(move |db_conn| {
            project_people::table
                .select(project_people::project_id)
                .filter(project_people::person_id.eq(user_id))
                .load(db_conn)
        });

        let (user_fields, user_projects) = tokio::try_join!(user_fields, user_projects)?;

        Ok(AuthorizationData {
            user_fields: user_fields?,
            projects: user_projects?.into_iter().collect(),
        })
    }

    pub fn id(&self) -> Uuid {
        self.sub
    }
}
