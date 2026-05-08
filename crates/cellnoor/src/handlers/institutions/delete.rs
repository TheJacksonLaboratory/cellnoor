use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{auth::AuthUser, db, error::Error, handlers::path::IdParam, state::AppState};

pub async fn delete_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let result = delete_institution_by_id(&tx, id).await.map(Json);

    tx.commit().await?;

    result
}

pub async fn delete_institution_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), Error> {
    let n = tx
        .execute("delete from institution where id = $1", &[&id])
        .await?;

    if n == 0 {
        return Err(Error::resource_not_found());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::{Error, ErrorInner},
        handlers::institutions::{
            create::{insert_institution, test::new_institution},
            delete::delete_institution_by_id,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let institution = insert_institution(&tx, &new_institution()).await.unwrap();
        delete_institution_by_id(&tx, institution.record.id)
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let Error { error } = delete_institution_by_id(&tx, Uuid::new_v4())
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
