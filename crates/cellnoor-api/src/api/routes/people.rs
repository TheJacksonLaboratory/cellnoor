use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::person::{
    Person, PersonQuery, PersonUpdate, PersonWithPermissions, SimplePersonQuery,
};

use crate::{
    auth::AuthUser,
    db::DbError,
    handlers::{
        delete_resource,
        permissions::{grant_permissions_to_principal, revoke_permissions_from_principal},
        people::{
            create_person, index_people, index_people_with_permissions, show_person,
            update_person,
        },
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_person).get(index_people_simple))
        .api_route("/permissions", get(index_people_with_permissions_simple))
        .api_route("/search", post(index_people))
        .api_route("/search/permissions", post(index_people_with_permissions))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    // Deleting a person fires a trigger that drops the principal, which
    // cascades to their accounts and keys
    ApiRouter::new()
        .api_route(
            "/",
            get(show_person)
                .put(update_person)
                .delete(delete_resource::<PersonUpdate>),
        )
        .api_route(
            "/permissions",
            post(grant_permissions_to_principal).delete(revoke_permissions_from_principal),
        )
}

async fn index_people_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimplePersonQuery>,
) -> Result<Json<Vec<Person>>, DbError> {
    index_people(
        state,
        user,
        crate::extract::JsonExtractor(PersonQuery::from_simple_query(q)),
    )
    .await
}

async fn index_people_with_permissions_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimplePersonQuery>,
) -> Result<Json<Vec<PersonWithPermissions>>, DbError> {
    index_people_with_permissions(
        state,
        user,
        crate::extract::JsonExtractor(PersonQuery::from_simple_query(q)),
    )
    .await
}
