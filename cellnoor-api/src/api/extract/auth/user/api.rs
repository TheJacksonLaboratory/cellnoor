use std::{
    collections::{HashMap, HashSet},
    ops::DerefMut,
};
use tokio_stream::StreamExt;

use cellnoor_schema::{people, project_people};
use diesel::{HasQuery, prelude::*};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::{self, DbConnection};

use super::common::*;

pub type User = StandardClaims;

impl User {
    pub(super) async fn authorization(
        &self,
        mut db_conn: &AsyncPgConnection,
    ) -> Result<Authorization, db::Error> {
        let user_id = self.id();

        let user_fields = UserFields::query()
            .filter(people::id.eq(user_id))
            .first(&mut db_conn);

        let user_projects = project_people::table
            .select(project_people::project_id)
            .filter(project_people::person_id.eq(user_id))
            .load_stream::<Uuid>(&mut db_conn);

        let (user_fields, mut user_projects) = tokio::try_join!(user_fields, user_projects)?;

        let mut projects = HashSet::with_capacity(500);
        while let Some(project_id) = user_projects.next().await {
            projects.insert(project_id?);
        }

        Ok(Authorization {
            user_fields,
            projects,
        })
    }

    pub fn id(&self) -> Uuid {
        self.sub
    }
}
