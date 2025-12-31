use axum::{extract::State, http::StatusCode};
use cellnoor_models::library::{Library, LibraryId};
use cellnoor_schema::libraries;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_library(
    request: LibraryId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Library> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Library> for LibraryId {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Library, db::Error> {
        Ok(Library::query()
            .filter(libraries::id.eq(self))
            .first(db_conn)?)
    }
}
