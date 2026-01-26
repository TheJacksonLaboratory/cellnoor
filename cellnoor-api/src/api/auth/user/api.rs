use serde::Deserialize;
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

use super::{FromEncodedJwt, common::*};

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct User(StandardClaims);

impl FromEncodedJwt for User {}

impl User {
    pub async fn with_authorized_projects(
        &self,
        mut db_conn: &AsyncPgConnection,
    ) -> Result<super::AuthenticatedUser, db::Error> {
        let user_id = self.0.sub;

        let user = PrivateClaims::query()
            .filter(people::id.eq(user_id))
            .first(&mut db_conn);

        let user_projects = project_people::table
            .select(project_people::project_id)
            .filter(project_people::person_id.eq(user_id))
            .load_stream::<Uuid>(&mut db_conn);

        let (user, mut user_projects) = tokio::try_join!(user, user_projects)?;

        let mut projects = HashSet::with_capacity(500);
        while let Some(project_id) = user_projects.next().await {
            projects.insert(project_id?);
        }

        Ok(super::AuthenticatedUser { user, projects })
    }
}
