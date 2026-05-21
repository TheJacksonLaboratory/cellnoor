use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    id::Id,
    suspension::{
        SavedSuspensionRecord, SavedSuspensionRecordDetailed, Suspension, SuspensionPredicate,
        SuspensionPredicateInner, SuspensionQuery,
    },
};
use futures::StreamExt;

use crate::{
    auth::AuthUser,
    db::{self, AsPredicate, BaseSqlStmt},
    error::{Error, ErrorInner},
    handlers::specimens::index::specimen_from_record,
    state::AppState,
};

pub async fn index_suspensions(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut query): Json<SuspensionQuery>,
) -> Result<Json<Vec<Suspension>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspensions(&tx, &mut query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_suspensions(
    tx: &db::Transaction<'_>,
    query: &mut SuspensionQuery,
) -> Result<Vec<Suspension>, ErrorInner> {
    let stmt = if query.detailed {
        include_str!("index/select_detailed.sql")
    } else {
        include_str!("index/select_compact.sql")
    };

    let sql = BaseSqlStmt::new(stmt).finish_with_query(query)?;

    let suspensions = if query.detailed {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(suspension_from_detailed_record).collect().await
    } else {
        let stream = tx.query_stream_into(sql).await?;
        stream.map(suspension_from_record).collect().await
    };

    Ok(suspensions)
}

fn suspension_simple_links(id: Id) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/suspensions", id)
}

pub fn suspension_from_record(record: SavedSuspensionRecord) -> Suspension {
    Suspension::Compact {
        links: suspension_simple_links(record.id),
        record,
    }
}

fn suspension_from_detailed_record(
    SavedSuspensionRecordDetailed {
        suspension,
        specimen,
        measurements,
        preparers,
    }: SavedSuspensionRecordDetailed,
) -> Suspension {
    Suspension::Detailed {
        links: suspension_simple_links(suspension.id),
        record: suspension,
        specimen: specimen_from_record(specimen),
        measurements,
        preparers,
    }
}

impl AsPredicate for SuspensionPredicate {
    fn as_predicate(
        &self,
    ) -> (
        &'static str,
        (&'static str, &(dyn postgres_types::ToSql + Sync)),
    ) {
        let sql = match self {
            Self::Specimen(p) => return p.as_predicate(),
            Self::Suspension(field) => match field {
                SuspensionPredicateInner::Id(u) | SuspensionPredicateInner::SpecimenId(u) => {
                    u.as_sql_operator_and_value()
                }
                SuspensionPredicateInner::ReadableId(s) => s.as_sql_operator_and_value(),
                SuspensionPredicateInner::Content(c) => c.as_sql_operator_and_value(),
                SuspensionPredicateInner::CreatedAt(t) => t.as_sql_operator_and_value(),
                SuspensionPredicateInner::LysisDurationMinutes(f) => f.as_sql_operator_and_value(),
                SuspensionPredicateInner::TargetCellRecovery(i) => i.as_sql_operator_and_value(),
                SuspensionPredicateInner::AdditionalData(j) => j.as_sql_operator_and_value(),
            },
        };

        (self.field_name(), sql)
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        suspension::{SuspensionPredicateInner, SuspensionQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        handlers::suspensions::{
            create::test::insert_test_suspension_and_specimen, index::select_suspensions,
        },
        state::test_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn select_with_filter() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap();

        let suspensions = select_suspensions(
            &tx,
            &mut SuspensionQuery::from_filter(
                SuspensionPredicateInner::Id(UuidOperator::Eq(*inserted.record().id)).into(),
                false,
            ),
        )
        .await
        .unwrap();

        assert_eq!(suspensions.len(), 1);
        assert_eq!(suspensions[0].record(), inserted.record());
    }
}
