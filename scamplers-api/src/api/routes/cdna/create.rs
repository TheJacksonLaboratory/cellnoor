use axum::{extract::State, http::StatusCode};
use diesel::{RunQueryDsl, prelude::*};
use scamplers_models::cdna::{Cdna, CdnaCreation, CdnaId};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_cdna(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<CdnaCreation>,
) -> ApiResponse<Cdna> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Cdna> for CdnaCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Cdna, db::Error> {
        use scamplers_schema::cdna::dsl::*;

        let created_id: CdnaId = diesel::insert_into(cdna)
            .values(self)
            .returning(id)
            .get_result(db_conn)?;

        created_id.execute(db_conn)
    }
}
