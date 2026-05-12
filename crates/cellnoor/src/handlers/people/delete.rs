use axum::{
    Json,
    extract::{Path, State},
};
use deadpool_postgres::tokio_postgres::error::SqlState;
use futures::TryFutureExt;
use postgres_types::ToSql;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    handlers::path::IdParam,
    state::AppState,
};

pub async fn delete_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = delete_person_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn delete_person_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    let params: [&(dyn ToSql + Sync); _] = [&id];
    let delete = tx
        .execute("delete from person where id = $1", &params)
        .map_err(ErrorInner::from);

    let (n, _) = tokio::try_join!(delete, drop_db_user(tx, id))?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound);
    }

    Ok(())
}

async fn drop_db_user(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    tx.acquire_user_permisssions_lock().await?;

    let revoke_result = tx
        .execute(
            &format!(r#"revoke all on all tables in schema public from "{id}""#),
            &[],
        )
        .await;

    if let Err(e) = revoke_result {
        let Some(&SqlState::UNDEFINED_OBJECT) = e.as_db_error().map(|inner| inner.code()) else {
            return Err(e.into());
        };

        return Err(ErrorInner::ResourceNotFound);
    }

    tx.execute(&format!(r#"drop user if exists "{}""#, id), &[])
        .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
        handlers::people::{
            create::{insert_person, test::new_person},
            delete::delete_person_by_id,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let person = insert_person(&tx, &new_person()).await.unwrap();
        delete_person_by_id(&tx, *person.record.id).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = delete_person_by_id(&tx, Uuid::new_v4()).await.unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
