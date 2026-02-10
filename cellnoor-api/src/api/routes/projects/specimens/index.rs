use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    specimen::{SpecimenQuery, SpecimenSummary},
};

use crate::{
    api::{extract::AuthJsonQuery, routes::specimens::index::select_specimens},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_project_specimens(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    AuthJsonQuery { mut q }: AuthJsonQuery<SpecimenQuery>,
) -> Result<Json<Vec<SpecimenSummary>>, db::Error> {
    q.filter.project_ids = Some(vec![id]);

    select_specimens(q, &db_conn).await.map(Json)
}
