use axum::{Json, extract::State, http::StatusCode};
use cellnoor_models::suspension::{NewNucleusSuspension, Suspension, SuspensionContent};
use cellnoor_schema::suspensions;
use diesel::prelude::*;

use crate::{db, state::AppState};

pub(super) async fn create_nucleus_suspension(
    _: State<AppState>,
    Json(request): Json<NewNucleusSuspension>,
) -> Result<Json<Suspension>> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Suspension> for NewNucleusSuspension {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Suspension, db::Error> {
        let preparer_ids = self.common().preparer_ids().to_vec();

        let suspension_id: SuspensionId = diesel::insert_into(suspensions::table)
            .values((self, suspensions::content.eq(SuspensionContent::Nuclei)))
            .returning(suspensions::id)
            .get_result(db_conn)?;

        insert_suspension_preparers(suspension_id, &preparer_ids, db_conn)?;

        suspension_id.execute(db_conn)
    }
}

#[derive(Debug, thiserror::Error, Serialize, JsonSchema, OperationIo)]
#[serde(rename_all = "snake_case", tag = "type")]
#[schemars(rename = "CreatePersonError")]
#[error(transparent)]
pub enum Error {
    Database(#[from] db::Error),
    #[error("invalid email")]
    CreatedBeforeSpecimen,
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Self::Database(err.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Database(e) => e.into_response(),
            Self::InvalidEmail => (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response(),
        }
    }
}
