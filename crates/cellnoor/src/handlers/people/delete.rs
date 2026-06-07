use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::IdParam,
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

async fn delete_person_by_id(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    // This has to come first because the database won't let you drop a user who
    // still has a row in the person table
    drop_db_user(tx, id).await?;
    db::delete_by_id(tx, "person", id).await?;

    Ok(())
}

async fn drop_db_user(tx: &db::Transaction<'_>, id: Uuid) -> Result<(), ErrorInner> {
    tx.execute_raw_sql("select drop_person_user($1)", &[&id])
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
