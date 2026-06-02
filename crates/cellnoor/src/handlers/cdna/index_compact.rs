use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    cdna::{CdnaCompact, CdnaPredicate, CdnaPredicateInner, CdnaQuery, SavedCdnaRecord},
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

pub async fn index_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<CdnaQuery>,
) -> Result<Json<Vec<CdnaCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_cdna_compact(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_cdna_compact(
    tx: &db::Transaction<'_>,
    query: &mut CdnaQuery,
) -> Result<Vec<CdnaCompact>, ErrorInner> {
    static SELECT_COMPACT_CDNA: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    let sql = SELECT_COMPACT_CDNA.finish_with_query(query);

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(cdna_from_record).collect().await)
}

impl AsPredicate for CdnaPredicate {
    fn as_predicate(&self) -> (&'static str, (&'static str, &(dyn ToSql + Sync))) {
        let sql = match self {
            Self::Specimen(p) => return p.as_predicate(),
            Self::Cdna(field) => match field {
                CdnaPredicateInner::Id(u) | CdnaPredicateInner::GemWellId(u) => {
                    u.as_sql_operator_and_value()
                }
                CdnaPredicateInner::ReadableId(s) => s.as_sql_operator_and_value(),
                CdnaPredicateInner::LibraryType(l) => l.as_sql_operator_and_value(),
                CdnaPredicateInner::PreparedAt(t) => t.as_sql_operator_and_value(),
                CdnaPredicateInner::NAmplificationCycles(i) => i.as_sql_operator_and_value(),
                CdnaPredicateInner::AdditionalData(j) => j.as_sql_operator_and_value(),
            },
        };

        (self.field_name(), sql)
    }
}

pub(super) fn cdna_simple_links(id: Id) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/cdna", id)
}

pub fn cdna_from_record(record: SavedCdnaRecord) -> CdnaCompact {
    CdnaCompact {
        links: cdna_simple_links(record.id),
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
            &mut CdnaQuery::from_filter(
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
