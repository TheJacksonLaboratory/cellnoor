use axum::{extract::State, http::StatusCode};
use diesel::RunQueryDsl;
use scamplers_models::library::{Library, LibraryCreation, LibraryId};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_library(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<LibraryCreation>,
) -> ApiResponse<Library> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Library> for LibraryCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Library, db::Error> {
        use scamplers_schema::libraries::dsl::*;

        let created_id: LibraryId = diesel::insert_into(libraries)
            .values(self)
            .returning(id)
            .get_result(db_conn)?;

        created_id.execute(db_conn)
    }
}
