use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    cdna::{CdnaCompact, CdnaQuery, SavedCdnaRecord},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<CdnaQuery>,
) -> Result<Json<Vec<CdnaCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_cdna_compact(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_cdna_compact(
    tx: &db::Transaction<'_>,
    query: &CdnaQuery,
) -> Result<Vec<CdnaCompact>, ErrorInner> {
    static SELECT_COMPACT_CDNA: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    Ok(tx
        .select(&SELECT_COMPACT_CDNA, query)
        .await?
        .into_iter()
        .map(cdna_from_record)
        .collect())
}

pub(super) fn cdna_simple_links(id: Uuid) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/cdna", id.into())
}

pub fn cdna_from_record(record: SavedCdnaRecord) -> CdnaCompact {
    CdnaCompact {
        links: cdna_simple_links(*record.id),
        record,
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        cdna::{CdnaField, CdnaPredicateInner, CdnaQuery},
        operator::UuidOperator,
        specimen::SpecimenField,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::cdna::{
            create::test::insert_test_cdna_and_chromium_run, index_compact::select_cdna_compact,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_cdna_and_chromium_run(&tx, |_| ())
            .await
            .unwrap();

        let cdnas = select_cdna_compact(
            &tx,
            &CdnaQuery::from_filter(
                CdnaPredicateInner::Id(UuidOperator::Eq(*inserted.record.id)).into(),
            ),
        )
        .await
        .unwrap();

        assert_eq!(cdnas.len(), 1);
        assert_eq!(cdnas[0].record, inserted.record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let view = "chromium_cdna_to_specimen";

        tokio::join!(
            ensure_fields_are_selectable::<SpecimenField>(&tx, view),
            ensure_fields_are_selectable::<CdnaField>(&tx, view),
        );
    }
}
