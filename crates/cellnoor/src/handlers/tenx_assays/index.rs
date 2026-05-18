use axum::{Json, extract::State};
use cellnoor_types::tenx_assay::TenxAssay;
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, SqlTemplate},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_tenx_assays(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<TenxAssay>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_tenx_assays(&tx).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_tenx_assays(tx: &db::Transaction<'_>) -> Result<Vec<TenxAssay>, ErrorInner> {
    let sql = SqlTemplate::new(include_str!("index/select.sql")).finish_with_params(vec![]);

    Ok(tx.query_stream_into(sql).await?.collect().await)
}

#[cfg(test)]
mod test {
    use crate::{
        handlers::tenx_assays::create::insert_test_chromium_assay,
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_chromium_assay(&tx).await.unwrap();
    }
}
