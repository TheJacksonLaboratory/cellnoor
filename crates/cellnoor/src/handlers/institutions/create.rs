use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, NewInstitution};

use crate::{auth::AuthUser, db, error::Error, state::AppState};

pub async fn create_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_institution(&tx, &institution).await.map(Json);

    tx.commit().await?;

    response
}

pub(super) async fn insert_institution(
    tx: &db::Transaction<'_>,
    NewInstitution {
        name,
        microsoft_entra_tenant_id,
    }: &NewInstitution,
) -> Result<Institution, crate::error::Error> {
    // Simple queries can be written inline
    let institution = tx
        .query_one_into_mapped(
            "insert into institution (name, microsoft_entra_tenant_id) values ($1, $2) returning \
             institution",
            &[name, microsoft_entra_tenant_id],
        )
        .await?;

    Ok(institution)
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::institution::NewInstitution;
    use uuid::Uuid;

    use crate::{
        handlers::institutions::create::insert_institution,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub fn new_institution() -> NewInstitution {
        NewInstitution {
            name: "institution".to_nonempty_string(),
            microsoft_entra_tenant_id: Uuid::new_v4(),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_institution(&tx, &new_institution()).await.unwrap();
    }
}
