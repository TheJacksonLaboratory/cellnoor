use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    cdna::{Cdna, CdnaField, CdnaPredicate, CdnaPredicateInner, CdnaQuery, SavedCdnaRecord},
    order_by::OrderBy,
    suspension_pool::{SavedTaggedSpecimenRecord, TaggedSpecimen},
};
use deadpool_postgres::tokio_postgres::Row;
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, BaseSqlStmt},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_cdna(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<CdnaQuery>,
) -> Result<Json<Vec<Cdna>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_cdna(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_cdna(
    tx: &db::Transaction<'_>,
    query: &mut CdnaQuery,
) -> Result<Vec<Cdna>, ErrorInner> {
    // The first column in the `order by` clause needs to match the `distinct on`
    // clause
    let distinct_on = OrderBy {
        field: CdnaField::Id,
        desc: true,
    };
    query.order_by.push_front(distinct_on);

    let stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = BaseSqlStmt::new(stmt).finish_with_query(query)?;

    let cdna = if query.detailed {
        let stream = tx.query_stream(sql).await?;
        stream
            .map(|row| row.map(map_detailed_row).unwrap())
            .collect()
            .await
    } else {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(Cdna::from_record).collect().await
    };

    Ok(cdna)
}

fn map_detailed_row(row: Row) -> Cdna {
    let record: SavedCdnaRecord = row.get("cdna");
    let specimens: Vec<SavedTaggedSpecimenRecord> = row.get("specimens");

    Cdna::Detailed {
        links: SimpleLinks::for_cdna(record.id),
        record,
        specimens: specimens
            .into_iter()
            .map(TaggedSpecimen::from_record)
            .collect(),
        measurements: row.get("measurements"),
        preparers: row.get("preparers"),
    }
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

#[cfg(test)]
mod test {
    use cellnoor_types::{
        cdna::{CdnaPredicateInner, CdnaQuery},
        operator::UuidOperator,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::cdna::{create::test::insert_test_cdna, index::select_cdna},
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_cdna(&tx, |_| ()).await.unwrap();

        let cdnas = select_cdna(
            &tx,
            &mut CdnaQuery::from_filter(
                CdnaPredicateInner::Id(UuidOperator::Eq(*inserted.record().id)).into(),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(cdnas.len(), 1);
        assert_eq!(cdnas[0].record(), inserted.record());
    }
}
