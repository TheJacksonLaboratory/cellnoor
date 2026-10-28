use aide::axum::{ApiRouter, routing::get};

use crate::{
    handlers::tenx_assays::{create_tenx_assay, index_tenx_assays},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_tenx_assays).post(create_tenx_assay))
}
