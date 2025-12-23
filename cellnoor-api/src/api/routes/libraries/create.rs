use axum::{extract::State, http::StatusCode};
use cellnoor_models::library::{Library, LibraryCreation, LibraryId};
use cellnoor_schema::library_preparers;
use diesel::prelude::*;
use uuid::Uuid;

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
        use cellnoor_schema::libraries::dsl::*;

        let preparer_ids = self.preparer_ids().to_vec();

        let library_id: LibraryId = diesel::insert_into(libraries)
            .values(self)
            .returning(id)
            .get_result(db_conn)?;

        insert_library_preparers(library_id, &preparer_ids, db_conn)?;

        library_id.execute(db_conn)
    }
}

fn insert_library_preparers(
    library_id: LibraryId,
    preparer_ids: &[Uuid],
    db_conn: &mut diesel::PgConnection,
) -> Result<(), db::Error> {
    let preparer_mappings: Vec<_> = preparer_ids
        .iter()
        .map(|p| {
            (
                library_preparers::library_id.eq(library_id),
                library_preparers::prepared_by.eq(p),
            )
        })
        .collect();

    diesel::insert_into(library_preparers::table)
        .values(preparer_mappings)
        .execute(db_conn)?;

    Ok(())
}
