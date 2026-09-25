use axum::{Json, extract::State};
use cellnoor_types::tenx_assay::TenxAssay;

use crate::{
    auth::AuthUser,
    db::{self, DbError, Sql},
    state::AppState,
};

pub async fn index_tenx_assays(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<TenxAssay>>, DbError> {
    state
        .in_transaction(user, async |tx| select_tenx_assays(tx).await)
        .await
}

async fn select_tenx_assays(tx: &db::Transaction<'_>) -> Result<Vec<TenxAssay>, DbError> {
    let sql = Sql::new(include_str!("index/select.sql"), vec![]);

    tx.query_into(&sql).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::tenx_assay::TenxAssayField;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::tenx_assays::create::insert_test_chromium_assay,
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_chromium_assay(&tx).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<TenxAssayField>(&tx, "tenx_assay").await;
    }
}
