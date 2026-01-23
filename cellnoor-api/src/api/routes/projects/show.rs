use axum::{extract::State, http::StatusCode};
use cellnoor_models::{
    institution::{Institution, InstitutionId},
    project::{Project, ProjectId},
};
use cellnoor_schema::projects::dsl::id;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{
    api::{
        self,
        auth::{self, Authorization},
        extract::auth::AuthenticatedUser,
        request::{AuthorizedRequest, Request},
    },
    db::{self},
    state::AppState,
};

impl AuthorizedRequest<Project> for ProjectId {
    type ValidationData = ();

    fn validate(&self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(self, mut db_conn: &AsyncPgConnection) -> Result<Project, api::Error> {
        Ok(Project::query()
            .filter(id.eq(self))
            .first(&mut db_conn)
            .await?)
    }
}

impl Request<Project> for ProjectId {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: &AsyncPgConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn authorize(self, authorization: Authorization) -> Result<Self, auth::Error> {
        let authorized_projects = authorization
            .authorized_projects(self.into())
            .expect("this should be `Some` because we are passing in a project");

        if !authorized_projects.contains(&self.into()) {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(self)
    }
}
