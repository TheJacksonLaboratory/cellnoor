use aide::axum::{ApiRouter, routing::get};
use axum::handler::Handler;

use crate::{api::middleware::admin_required, state::AppState};

use add::add_person_to_project;
use index::index_project_people;
use remove::remove_person_from_project;

mod add;
mod index;
mod remove;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(index_project_people)
            .patch(add_person_to_project)
            .delete(remove_person_from_project.layer(axum::middleware::from_fn(admin_required))),
    )
}
