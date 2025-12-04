use axum::{extract::State, http::StatusCode};
use diesel::{PgConnection, prelude::*};
use scamplers_models::cdna::{Cdna, CdnaId};
use scamplers_schema::cdna;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_cdna(
    request: CdnaId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Cdna> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Cdna> for CdnaId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Cdna, db::Error> {
        Ok(Cdna::query().filter(cdna::id.eq(self)).first(db_conn)?)
    }
}
