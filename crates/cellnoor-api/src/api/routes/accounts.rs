use aide::axum::{ApiRouter, routing::get};

use crate::{
    handlers::accounts::{create_account, index_accounts},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_accounts).post(create_account))
}
