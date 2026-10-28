use aide::axum::{
    ApiRouter,
    routing::{get, post, put},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::bot_account::{
    BotAccount, BotAccountDetailed, BotAccountQuery, BotAccountSimpleFields, SimpleBotAccountQuery,
};

use crate::{
    auth::AuthUser,
    db::DbError,
    handlers::{
        bot_accounts::{
            add_people_to_bot_account, create_bot_account, index_bot_accounts,
            index_bot_accounts_detailed, remove_people_from_bot_account, update_bot_account,
        },
        delete_resource,
        permissions::{grant_permissions_to_principal, revoke_permissions_from_principal},
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_bot_account).get(index_bot_accounts_simple))
        .api_route("/detailed", get(index_bot_accounts_detailed_simple))
        .api_route("/search", post(index_bot_accounts))
        .api_route("/search/detailed", post(index_bot_accounts_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    // Deleting a bot_account fires a trigger that drops the principal, which
    // cascades to who has access to the bot_account and to its API keys
    ApiRouter::new()
        .api_route(
            "/",
            put(update_bot_account).delete(delete_resource::<BotAccountSimpleFields>),
        )
        .api_route(
            "/people",
            post(add_people_to_bot_account).delete(remove_people_from_bot_account),
        )
        .api_route(
            "/permissions",
            post(grant_permissions_to_principal).delete(revoke_permissions_from_principal),
        )
}

async fn index_bot_accounts_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleBotAccountQuery>,
) -> Result<Json<Vec<BotAccount>>, DbError> {
    index_bot_accounts(
        state,
        user,
        crate::extract::JsonExtractor(BotAccountQuery::from_simple_query(q)),
    )
    .await
}

async fn index_bot_accounts_detailed_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleBotAccountQuery>,
) -> Result<Json<Vec<BotAccountDetailed>>, DbError> {
    index_bot_accounts_detailed(
        state,
        user,
        crate::extract::JsonExtractor(BotAccountQuery::from_simple_query(q)),
    )
    .await
}
