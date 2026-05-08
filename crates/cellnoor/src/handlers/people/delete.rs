use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{auth::AuthUser, db, error::Error, handlers::path::IdParam, state::AppState};

pub async fn delete_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let result = delete_person_by_id(&tx, id).await.map(Json);

    tx.commit().await?;

    result
}

pub async fn delete_person_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), Error> {
    let n = tx
        .execute("delete from person where id = $1", &[&id])
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
        handlers::people::{
            create::{insert_person, test::new_person},
            delete::delete_person_by_id,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_existing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let person = insert_person(&tx, &new_person()).await.unwrap();
        delete_person_by_id(&tx, person.record.id).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let Error { error } = delete_person_by_id(&tx, Uuid::new_v4())
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
