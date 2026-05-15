use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::institution::{Institution, NewInstitution};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{institutions::show::select_institution_by_id, path::IdParam},
    state::AppState,
};

pub async fn update_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_institution_by_id(&tx, id, &institution)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_institution_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    updated_record: &NewInstitution,
) -> Result<Institution, ErrorInner> {
    db::update(tx, "institution", id, updated_record).await?;

    select_institution_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use cellnoor_types::institution::{Institution, SavedInstitutionRecord};

    use crate::{
        handlers::institutions::{
            create::test::insert_test_institution, update::update_institution_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            mut pre_update,
            Institution {
                record: SavedInstitutionRecord { id, .. },
                links: _,
            },
        ) = insert_test_institution(&tx, identity).await;
        pre_update.name = "updated".to_nonempty_string();

        update_institution_by_id(&tx, *id, &pre_update)
            .await
            .unwrap();
    }
}
