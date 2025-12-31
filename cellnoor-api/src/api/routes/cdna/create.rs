use axum::{extract::State, http::StatusCode};
use cellnoor_models::cdna::{Cdna, CdnaCreation, CdnaId};
use cellnoor_schema::cdna_preparers;
use diesel::{RunQueryDsl, prelude::*};
use uuid::Uuid;

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
        use cellnoor_schema::cdna::dsl::*;

        let preparer_ids = self.preparer_ids().to_vec();

        let cdna_id: CdnaId = diesel::insert_into(cdna)
            .values(self)
            .returning(id)
            .get_result(db_conn)?;

        insert_cdna_preparers(cdna_id, &preparer_ids, db_conn)?;

        cdna_id.execute(db_conn)
    }
}

fn insert_cdna_preparers(
    cdna_id: CdnaId,
    preparer_ids: &[Uuid],
    db_conn: &mut diesel::PgConnection,
) -> Result<(), db::Error> {
    let preparer_mappings: Vec<_> = preparer_ids
        .iter()
        .map(|p| {
            (
                cdna_preparers::cdna_id.eq(cdna_id),
                cdna_preparers::prepared_by.eq(p),
            )
        })
        .collect();

    diesel::insert_into(cdna_preparers::table)
        .values(preparer_mappings)
        .execute(db_conn)?;

    Ok(())
}
