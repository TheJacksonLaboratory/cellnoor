use axum::{Json, extract::State};
use cellnoor_types::multiplexing_tag::{MultiplexingTag, NewMultiplexingTag};
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    state::AppState,
};

pub async fn create_multiplexing_tag(
    State(state): State<AppState>,
    user: AuthUser,
    Json(new): Json<NewMultiplexingTag>,
) -> Result<Json<MultiplexingTag>, DbError> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = insert_multiplexing_tag(&tx, &new).await?;

    tx.commit().await?;

    Ok(Json(response))
}

async fn insert_multiplexing_tag(
    tx: &db::Transaction<'_>,
    NewMultiplexingTag { tag_id, type_ }: &NewMultiplexingTag,
) -> Result<MultiplexingTag, DbError> {
    static INSERT_MULTIPLEXING_TAG: &str =
        "insert into multiplexing_tag (tag_id, type) values ($1, $2) returning multiplexing_tag";

    let params: Vec<&(dyn ToSql + Sync)> = vec![tag_id, type_];

    tx.query_one_into(&Sql::new(INSERT_MULTIPLEXING_TAG, params))
        .await
}

#[cfg(test)]
pub mod tests {
    use cellnoor_types::{
        multiplexing_tag::{MultiplexingTag, NewMultiplexingTag},
        suspension_pool::MultiplexingTagType,
    };
    use uuid::Uuid;

    use crate::{
        db::{self, DbError},
        handlers::multiplexing_tags::create::insert_multiplexing_tag,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_multiplexing_tag(
        tx: &db::Transaction<'_>,
    ) -> Result<MultiplexingTag, DbError> {
        let new = NewMultiplexingTag {
            tag_id: Uuid::new_v4().to_string().to_nonempty_string(),
            type_: MultiplexingTagType::FlexBarcode,
        };

        insert_multiplexing_tag(tx, &new).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_multiplexing_tag(&tx).await.unwrap();
    }
}
