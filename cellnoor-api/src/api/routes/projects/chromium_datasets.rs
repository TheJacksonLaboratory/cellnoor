use aide::axum::ApiRouter;

use crate::state::AppState;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
}
