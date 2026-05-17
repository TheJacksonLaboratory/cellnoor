use axum::{
    Json,
    extract::{Path, State},
};
use deadpool_postgres::tokio_postgres::error::SqlState;
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
    tokio::try_join!(db::delete_by_id(tx, "person", id), drop_db_user(tx, id))?;

    Ok(())
}

async fn drop_db_user(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    tx.acquire_user_permisssions_lock().await?;

    let revoke_result = tx
        .execute_raw_sql(
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

    tx.execute_raw_sql(&format!(r#"drop user if exists "{}""#, id), &[])
        .await?;

    Ok(())
}

#[cfg(test)]
mod test {

    use crate::{
        handlers::people::{
            create::test::insert_test_person_and_institution, delete::delete_person_by_id,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, person) = insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
        delete_person_by_id(&tx, *person.record.id).await.unwrap();
    }
}
