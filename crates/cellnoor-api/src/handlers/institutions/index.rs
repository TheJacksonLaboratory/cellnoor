use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    institution::{Institution, InstitutionQuery, SavedInstitutionRecord},
};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_institutions(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(query): crate::extract::JsonExtractor<InstitutionQuery>,
) -> Result<Json<Vec<Institution>>, DbError> {
    state
        .in_transaction(user, async |tx| select_institutions(tx, &query).await)
        .await
}

pub(in super::super) async fn select_institutions(
    tx: &db::Transaction<'_>,
    query: &InstitutionQuery,
) -> Result<Vec<Institution>, DbError> {
    static SELECT_INSTITUTIONS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select.sql"));

    Ok(tx
        .select(&SELECT_INSTITUTIONS, query)
        .await?
        .into_iter()
        .map(institution_from_record)
        .collect())
}

fn institution_from_record(record: SavedInstitutionRecord) -> Institution {
    Institution {
        links: SimpleLinks::from_str_and_id("/institutions", record.id),
        record,
    }
}

#[cfg(test)]
mod test {

    use cellnoor_types::{
        institution::{InstitutionField, InstitutionPredicate, InstitutionQuery, NewInstitution},
        operator::StringOperator,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
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
            &InstitutionQuery::from_filter(InstitutionPredicate::Name(StringOperator::Like(
                name.into(),
            ))),
        )
        .await
        .unwrap();

        assert_eq!(selected_records.len(), 1);
        assert_eq!(selected_records[0], inserted);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<InstitutionField>(&tx, "institution").await;
    }
}
