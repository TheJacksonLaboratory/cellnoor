use axum::{Json, extract::State};
use cellnoor_types::multiplexing_tag::MultiplexingTag;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    state::AppState,
};

pub async fn index_multiplexing_tags(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<MultiplexingTag>>, DbError> {
    state
        .in_transaction(user, async |tx| select_multiplexing_tags(tx).await)
        .await
}

pub async fn select_multiplexing_tags(
    tx: &db::Transaction<'_>,
) -> Result<Vec<MultiplexingTag>, DbError> {
    static SELECT_MULTIPLEXING_TAGS: &str = include_str!("index/select.sql");

    let sql = Sql::new(SELECT_MULTIPLEXING_TAGS, vec![]);

    tx.query_into(&sql).await
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::multiplexing_tags::{
            create::tests::insert_test_multiplexing_tag, index::select_multiplexing_tags,
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_multiplexing_tag(&tx).await.unwrap();
        select_multiplexing_tags(&tx).await.unwrap();
    }
}
