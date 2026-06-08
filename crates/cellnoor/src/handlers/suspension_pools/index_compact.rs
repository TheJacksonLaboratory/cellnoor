use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    id::Id,
    suspension_pool::{
        MultiplexingTagPredicate, SavedSuspensionPoolRecord, SavedTaggedSpecimenRecord,
        SuspensionPoolCompact, SuspensionPoolLinks, SuspensionPoolPredicate,
        SuspensionPoolPredicateInner, SuspensionPoolQuery, TaggedSpecimen,
    },
};
use futures::StreamExt;
use postgres_types::ToSql;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    handlers::specimens::index_compact::specimen_from_record,
    state::AppState,
};

pub async fn index_suspension_pools(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SuspensionPoolQuery>,
) -> Result<Json<Vec<SuspensionPoolCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspension_pools_compact(&tx, &mut query)
        .await
        .map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_suspension_pools_compact(
    tx: &db::Transaction<'_>,
    query: &mut SuspensionPoolQuery,
) -> Result<Vec<SuspensionPoolCompact>, ErrorInner> {
    static SELECT_COMPACT_SUSPENSION_POOL: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    let sql = SELECT_COMPACT_SUSPENSION_POOL.finish_with_query(query);

    let stream = tx.query_stream_into(sql).await?;
    Ok(stream.map(suspension_pool_from_record).collect().await)
}

impl AsPredicate for MultiplexingTagPredicate {
    fn as_predicate(&self) -> (&'static str, (&'static str, &(dyn ToSql + Sync))) {
        let sql = match self {
            Self::Type(t) => t.as_sql_operator_and_value(),
        };

        (self.field_name(), sql)
    }
}

impl AsPredicate for SuspensionPoolPredicate {
    fn as_predicate(&self) -> (&'static str, (&'static str, &(dyn ToSql + Sync))) {
        let sql = match self {
            Self::Specimen(p) => {
                return p.as_predicate();
            }
            Self::MultiplexingTag(p) => {
                return p.as_predicate();
            }
            Self::SuspensionPool(p) => match p {
                SuspensionPoolPredicateInner::Id(u) => u.as_sql_operator_and_value(),
                SuspensionPoolPredicateInner::Name(s)
                | SuspensionPoolPredicateInner::ReadableId(s) => s.as_sql_operator_and_value(),

                SuspensionPoolPredicateInner::PooledAt(t) => t.as_sql_operator_and_value(),
                SuspensionPoolPredicateInner::AdditionalData(j) => j.as_sql_operator_and_value(),
            },
        };

        (self.field_name(), sql)
    }
}

pub(super) fn suspension_pool_links(id: Id) -> SuspensionPoolLinks {
    SuspensionPoolLinks {
        simple: SimpleLinks::from_str_and_id("/suspension-pools", id),
        suspensions: format!("/suspension-pools/{id}/suspensions"),
    }
}

pub fn suspension_pool_from_record(record: SavedSuspensionPoolRecord) -> SuspensionPoolCompact {
    SuspensionPoolCompact {
        links: suspension_pool_links(record.id.into()),
        record,
    }
}

pub fn tagged_specimen_from_record(
    SavedTaggedSpecimenRecord {
        specimen,
        multiplexing_tag,
        ocm_barcode_id,
    }: SavedTaggedSpecimenRecord,
) -> TaggedSpecimen {
    TaggedSpecimen {
        specimen: specimen_from_record(specimen),
        multiplexing_tag,
        ocm_barcode_id,
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::SimpleStringOperator,
        specimen::{SpecimenField, SpecimenPredicate},
        suspension_pool::{MultiplexingTagField, SuspensionPoolField, SuspensionPoolQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::suspension_pools::{
            create::test::insert_test_suspension_pool_and_suspensions,
            index_compact::select_suspension_pools_compact,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_specimen_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
            .await
            .unwrap();

        let mut query = SuspensionPoolQuery::from_filter(
            SpecimenPredicate::Name(
                SimpleStringOperator::Eq(inserted.specimens[0].specimen.record.name.clone().into())
                    .into(),
            )
            .into(),
        );

        let selected_suspension_pools = select_suspension_pools_compact(&tx, &mut query)
            .await
            .unwrap();

        assert_eq!(selected_suspension_pools.len(), 1);
        assert_eq!(selected_suspension_pools[0].record, inserted.record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let view = "suspension_pool_to_specimen";

        tokio::join!(
            ensure_fields_are_selectable::<SpecimenField>(&tx, view),
            ensure_fields_are_selectable::<MultiplexingTagField>(&tx, view),
            ensure_fields_are_selectable::<SuspensionPoolField>(&tx, view),
        );
    }
}
