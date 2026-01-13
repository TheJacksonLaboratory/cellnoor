use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::cdna::{Cdna, CdnaId};
use cellnoor_schema::cdna;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, handle_request},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_cdna(
    request: CdnaId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Cdna> {
    let item = handle_request(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Cdna> for CdnaId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Cdna, db::Error> {
        Ok(Cdna::query().filter(cdna::id.eq(self)).first(db_conn)?)
    }
}
