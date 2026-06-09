use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::library::{LibraryCompact, LibraryDetailed, LibraryQuery, SimpleLibraryQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::libraries::{
        create_library, create_library_measurement, delete_library, index_libraries,
        index_libraries_detailed, show_library, update_library,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_library).get(index_libraries_simple))
        .api_route("/detailed", get(index_libraries_detailed_simple))
        .api_route("/search", post(index_libraries))
        .api_route("/search/detailed", post(index_libraries_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get(show_library).put(update_library).delete(delete_library),
        )
        .api_route("/measurements", post(create_library_measurement))
}

async fn index_libraries_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleLibraryQuery>,
) -> Result<Json<Vec<LibraryCompact>>, Error> {
    index_libraries(state, user, Json(LibraryQuery::from_simple_query(q))).await
}

async fn index_libraries_detailed_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleLibraryQuery>,
) -> Result<Json<Vec<LibraryDetailed>>, Error> {
    index_libraries_detailed(state, user, Json(LibraryQuery::from_simple_query(q))).await
}
