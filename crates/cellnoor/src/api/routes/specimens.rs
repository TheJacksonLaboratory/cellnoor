use aide::axum::ApiRouter;

use crate::state::AppState;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
}
