use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use cellnoor_models::chromium_dataset::metrics::ParsedMetricsData;
use cellnoor_schema::chromium_dataset_parsed_files;
use create::create_chromium_dataset;
use diesel::{pg::Pg, prelude::*};
use index::index_chromium_datasets;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use show::show_chromium_dataset;
use uuid::Uuid;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod files;
pub mod index;
pub mod libraries;
pub mod parsed_files;
pub mod show;
pub mod specimens;

// We put these two structs here so that both `parsed_files` and `files` can
// access them
#[derive(Deserialize, JsonSchema)]
#[schemars(inline)]
struct FilePath {
    // This field has to be called `id` instead of `dataset_id` because there are other routes that
    // depend on the struct `IdParameter`, whose only field is called `id`
    id: Uuid,
    path: String,
}

#[derive(Insertable, AsChangeset, Identifiable, HasQuery, Serialize, JsonSchema)]
#[diesel(table_name = chromium_dataset_parsed_files, check_for_backend(Pg), primary_key(dataset_id, path))]
struct ParsedChromiumDatasetFile {
    dataset_id: Uuid,
    path: String,
    data: ParsedMetricsData,
}

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_chromium_dataset.layer(admin_required_creation!()))
                .get(index_chromium_datasets),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_chromium_dataset))
        .nest("/specimens", specimens::router())
        .nest("/libraries", libraries::router())
        .merge(files::router())
        .merge(parsed_files::router())
}
