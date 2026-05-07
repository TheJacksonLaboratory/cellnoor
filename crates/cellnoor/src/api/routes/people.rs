use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleQuery,
    person::{Person, PersonQuery, PersonSortField},
};
use serde_qs::web::QsQuery;

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::people::{create::create_person, index::index_people, show::show_person},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_person).get(index_people_simple))
        .api_route("/search", post(index_people))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_person))
}

async fn index_people_simple(
    state: State<AppState>,
    user: AuthUser,
    QsQuery(q): QsQuery<SimpleQuery<PersonSortField>>,
) -> Result<Json<Vec<Person>>, Error> {
    index_people(state, user, Json(PersonQuery::from_simple_query(q))).await
}
