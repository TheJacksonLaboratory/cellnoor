use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    chromium_dataset::{
        ChromiumDatasetCompact, ChromiumDatasetPredicate, ChromiumDatasetPredicateInner,
        ChromiumDatasetQuery, SavedChromiumDatasetRecord,
    },
    id::Id,
};
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_chromium_datasets(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<ChromiumDatasetQuery>,
) -> Result<Json<Vec<ChromiumDatasetCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_chromium_datasets_compact(&tx, &mut query)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_chromium_datasets_compact(
    tx: &db::Transaction<'_>,
    query: &mut ChromiumDatasetQuery,
) -> Result<Vec<ChromiumDatasetCompact>, ErrorInner> {
    static SELECT_COMPACT_CHROMIUM_DATASETS: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    let sql = SELECT_COMPACT_CHROMIUM_DATASETS.finish_with_query(query);

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(chromium_dataset_from_record).collect().await)
}

impl AsPredicate for ChromiumDatasetPredicate {
    fn as_predicate(&self) -> (&str, (&'static str, &(dyn ToSql + Sync))) {
        let sql = match self {
            Self::Specimen(p) => return p.as_predicate(),
            Self::TenxAssay(p) => return p.as_predicate(),
            Self::Library(p) => return p.as_predicate(),
            Self::ChromiumDataset(field) => match field {
                ChromiumDatasetPredicateInner::Id(u) => u.as_sql_operator_and_value(),
                ChromiumDatasetPredicateInner::Name(s) => s.as_sql_operator_and_value(),
                ChromiumDatasetPredicateInner::DeliveredAt(t) => t.as_sql_operator_and_value(),
            },
        };

        (self.field_name(), sql)
    }
}

pub(super) fn chromium_dataset_links(id: Id) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/chromium-datasets", id)
}

pub fn chromium_dataset_from_record(record: SavedChromiumDatasetRecord) -> ChromiumDatasetCompact {
    ChromiumDatasetCompact {
        links: chromium_dataset_links(record.id),
        record,
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        chromium_dataset::{
            ChromiumDatasetField, ChromiumDatasetPredicateInner, ChromiumDatasetQuery,
        },
        library::LibraryField,
        operator::UuidOperator,
        specimen::SpecimenField,
        tenx_assay::TenxAssayField,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::chromium_datasets::{
            create::test::insert_test_chromium_dataset,
            index_compact::select_chromium_datasets_compact,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_chromium_dataset(&tx, |_| ()).await.unwrap();

        let datasets = select_chromium_datasets_compact(
            &tx,
            &mut ChromiumDatasetQuery::from_filter(
                ChromiumDatasetPredicateInner::Id(UuidOperator::Eq(*inserted.record.id)).into(),
            ),
        )
        .await
        .unwrap();

        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].record, inserted.record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let view = "chromium_dataset_to_specimen";

        tokio::join!(
            ensure_fields_are_selectable::<SpecimenField>(&tx, view),
            ensure_fields_are_selectable::<TenxAssayField>(&tx, view),
            ensure_fields_are_selectable::<LibraryField>(&tx, view),
            ensure_fields_are_selectable::<ChromiumDatasetField>(&tx, view),
        );
    }
}
