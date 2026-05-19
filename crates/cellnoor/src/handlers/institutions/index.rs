use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, InstitutionPredicate, InstitutionQuery};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, BaseSqlStmt},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_institutions(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<InstitutionQuery>,
) -> Result<Json<Vec<Institution>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_institutions(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_institutions(
    tx: &db::Transaction<'_>,
    query: &mut InstitutionQuery,
) -> Result<Vec<Institution>, ErrorInner> {
    let sql = BaseSqlStmt::new(include_str!("index/select.sql")).finish_with_query(query)?;

    Ok(tx
        .query_stream_into(sql)
        .await
        .map(async |stream| stream.map(Institution::from_record).collect().await)?
        .await)
}

impl AsPredicate for InstitutionPredicate {
    fn as_predicate(&self) -> (&str, (&str, &(dyn postgres_types::ToSql + Sync))) {
        let field_name = self.as_ref();

        match self {
            Self::Id(u) | Self::MicrosoftEntraTenantId(u) => {
                (field_name, u.as_sql_operator_and_value())
            }
            Self::Name(s) => (field_name, s.as_sql_operator_and_value()),
        }
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        institution::{InstitutionPredicate, InstitutionQuery, NewInstitution},
        operator::StringOperator,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::institutions::{
            create::test::insert_test_institution, index::select_institutions,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (NewInstitution { name, .. }, inserted) =
            insert_test_institution(&tx, |_| ()).await.unwrap();

        let selected_records = select_institutions(
            &tx,
            &mut InstitutionQuery::from_filter(
                InstitutionPredicate::Name(StringOperator::Like(name.into())),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(selected_records.len(), 1);
        assert_eq!(selected_records[0], inserted);
    }
}
