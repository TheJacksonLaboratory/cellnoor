use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    chromium_dataset::{ChromiumDataset, ChromiumDatasetQuery},
    specimen::SpecimenFilter,
};

use crate::{
    api::{extract::AuthJsonQuery, routes::chromium_datasets::index::select_chromium_datasets},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_specimen_chromium_datasets(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    AuthJsonQuery { mut q }: AuthJsonQuery<ChromiumDatasetQuery>,
) -> Result<Json<Vec<ChromiumDataset>>, db::Error> {
    q.filter.specimen = Some(SpecimenFilter {
        ids: Some(vec![id]),
        ..Default::default()
    });

    select_chromium_datasets(q, &db_conn).await.map(Json)
}
