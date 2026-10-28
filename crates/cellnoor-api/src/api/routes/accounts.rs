use aide::axum::{ApiRouter, routing::get};

use crate::{handlers::accounts::index_accounts, state::AppState};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_accounts))
}
