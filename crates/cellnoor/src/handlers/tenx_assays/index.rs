use axum::{Json, extract::State};
use cellnoor_types::tenx_assay::{TenxAssay, TenxAssayPredicate};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, BaseSqlStmt},
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
    let sql = BaseSqlStmt::new(include_str!("index/select.sql")).finish_with_params(vec![]);

    Ok(tx.query_stream_into(sql).await?.collect().await)
}

impl AsPredicate for TenxAssayPredicate {
    fn as_predicate(
        &self,
    ) -> (
        &'static str,
        (&'static str, &(dyn postgres_types::ToSql + Sync)),
    ) {
        let sql = match self {
            Self::Id(u) => u.as_sql_operator_and_value(),
            Self::Name(s) | Self::ChemistryVersion(s) | Self::ProtocolUrl(s) => {
                s.as_sql_operator_and_value()
            }
        };

        (self.field_name(), sql)
    }
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
