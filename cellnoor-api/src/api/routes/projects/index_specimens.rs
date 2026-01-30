use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    specimen::{SpecimenQuery, SpecimenSummary},
};

use crate::{
    api::{extract::JsonQuery, routes::specimens::index::select_specimens},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_project_specimens(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    JsonQuery { mut q }: JsonQuery<SpecimenQuery>,
) -> Result<Json<Vec<SpecimenSummary>>, db::Error> {
    q.filter.projects = Some(vec![id]);

    select_specimens(q, &mut db_conn).await.map(Json)
}
