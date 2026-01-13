use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::library::{Library, LibraryId};
use cellnoor_schema::libraries;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, handle_request},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_library(
    request: LibraryId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Library> {
    let item = handle_request(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Library> for LibraryId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Library, db::Error> {
        Ok(Library::query()
            .filter(libraries::id.eq(self))
            .first(db_conn)?)
    }
}
