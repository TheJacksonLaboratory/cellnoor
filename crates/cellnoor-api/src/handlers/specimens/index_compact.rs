use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    specimen::{SavedSpecimenRecord, SpecimenCompact, SpecimenQuery},
};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FilterableSqlBuilder},
    state::AppState,
};

pub async fn index_specimens(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SpecimenQuery>,
) -> Result<Json<Vec<SpecimenCompact>>, DbError> {
    state
        .in_transaction(user, async |tx| select_specimens_compact(tx, &query).await)
        .await
}

async fn select_specimens_compact(
    tx: &db::Transaction<'_>,
    query: &SpecimenQuery,
) -> Result<Vec<SpecimenCompact>, DbError> {
    static SELECT_COMPACT_SPECIMEN: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    Ok(tx
        .select(&SELECT_COMPACT_SPECIMEN, query)
        .await?
        .into_iter()
        .map(specimen_from_record)
        .collect())
}

pub fn specimen_from_record(record: SavedSpecimenRecord) -> SpecimenCompact {
    SpecimenCompact {
        links: SimpleLinks::from_str_and_id("/specimens", record.id),
        record,
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        specimen::{SpecimenField, SpecimenPredicate, SpecimenQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::specimens::{
            create::test::insert_test_specimen_and_project, index_compact::select_specimens_compact,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();

        let specimens = select_specimens_compact(
            &tx,
            &SpecimenQuery::from_filter(SpecimenPredicate::Id(UuidOperator::Eq(
                *inserted.record.id,
            ))),
        )
        .await
        .unwrap();

        assert_eq!(specimens.len(), 1);
        assert_eq!(specimens[0].record, inserted.record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        ensure_fields_are_selectable::<SpecimenField>(&tx, "specimen").await;
    }
}
