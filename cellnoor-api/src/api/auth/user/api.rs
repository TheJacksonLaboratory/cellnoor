use std::{collections::HashSet, sync::Arc};

use cellnoor_schema::{people, project_people};
use diesel::{HasQuery, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;
use tokio_stream::StreamExt;
use uuid::Uuid;

use super::{AuthProjects, AuthUser, FromEncodedJwt, common::*};
use crate::db::{self};

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct User(StandardClaims);

impl FromEncodedJwt for User {}

impl User {
    pub(super) async fn with_authorized_projects(
        &self,
        mut db_conn: &AsyncPgConnection,
    ) -> Result<AuthUser, db::Error> {
        let user_id = self.0.sub;

        let user = PrivateClaims::query()
            .filter(people::id.eq(user_id))
            .first(&mut db_conn);

        let user_projects = project_people::table
            .select(project_people::project_id)
            .filter(project_people::person_id.eq(user_id))
            .load_stream::<Uuid>(&mut db_conn);

        let (user, mut user_projects) = tokio::try_join!(user, user_projects)?;

        if user.is_admin || user.is_biology_staff || user.is_computational_staff {
            return Ok(AuthUser {
                user,
                projects: AuthProjects::All,
            });
        }

        let mut projects = HashSet::with_capacity(500);
        while let Some(project_id) = user_projects.next().await {
            projects.insert(project_id?);
        }

        Ok(AuthUser {
            user,
            projects: AuthProjects::Some {
                project_ids: Arc::new(projects),
            },
        })
    }
}
