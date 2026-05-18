use axum::{Json, extract::State};
use cellnoor_types::multiplexing_tag::MultiplexingTag;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, SqlTemplate},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_multiplexing_tags(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<MultiplexingTag>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_multiplexing_tags(&tx).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_multiplexing_tags(
    tx: &db::Transaction<'_>,
) -> Result<Vec<MultiplexingTag>, ErrorInner> {
    let sql = SqlTemplate::new(include_str!("index/select.sql")).finish_with_params(vec![]);

    Ok(tx.query_stream_into(sql).await?.collect().await)
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::multiplexing_tags::{
            create::tests::insert_test_multiplexing_tag, index::select_multiplexing_tags,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test]
    async fn select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_multiplexing_tag(&tx).await.unwrap();
        select_multiplexing_tags(&tx).await.unwrap();
    }
}
