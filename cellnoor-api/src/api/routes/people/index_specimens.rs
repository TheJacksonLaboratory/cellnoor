use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    specimen::{SpecimenQuery, SpecimenSummary},
};

use crate::{
    api::{
        auth::{AuthenticatedUser, RemoveUnauthorizedProjects},
        extract::JsonQuery,
        routes::specimens::index::select_specimens,
    },
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_person_specimens(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthenticatedUser>,
    Path(IdParameter { id }): Path<IdParameter>,
    JsonQuery { mut q }: JsonQuery<SpecimenQuery>,
) -> Result<Json<Vec<SpecimenSummary>>, db::Error> {
    q.filter.projects.remove_unauthorized_projects(&user);
    q.filter.submitted_by = Some(vec![id]);

    select_specimens(q, &mut db_conn).await.map(Json)
}
