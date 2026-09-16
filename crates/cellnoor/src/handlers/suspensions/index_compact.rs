use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleLinks,
    id::Id,
    suspension::{SavedSuspensionRecord, SuspensionCompact, SuspensionQuery},
};

use crate::{
    auth::AuthUser,
    db::{self, FilterableSqlBuilder},
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn index_suspensions(
    State(state): State<AppState>,
    user: AuthUser,
    Json(query): Json<SuspensionQuery>,
) -> Result<Json<Vec<SuspensionCompact>>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_suspensions_compact(&tx, &query).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn select_suspensions_compact(
    tx: &db::Transaction<'_>,
    query: &SuspensionQuery,
) -> Result<Vec<SuspensionCompact>, ErrorInner> {
    static SELECT_COMPACT_SUSPENSION: FilterableSqlBuilder =
        FilterableSqlBuilder::new(include_str!("index/select_compact.sql"));

    Ok(tx
        .select(&SELECT_COMPACT_SUSPENSION, query)
        .await?
        .into_iter()
        .map(suspension_from_record)
        .collect())
}

pub(super) fn suspension_simple_links(id: Id) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/suspensions", id)
}

pub fn suspension_from_record(record: SavedSuspensionRecord) -> SuspensionCompact {
    SuspensionCompact {
        links: suspension_simple_links(record.id),
        record,
    }
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        operator::UuidOperator,
        specimen::SpecimenField,
        suspension::{SuspensionField, SuspensionPredicateInner, SuspensionQuery},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        db::test_utils::ensure_fields_are_selectable,
        handlers::suspensions::{
            create::test::insert_test_suspension_and_specimen,
            index_compact::select_suspensions_compact,
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

        let suspensions = select_suspensions_compact(
            &tx,
            &SuspensionQuery::from_filter(
                SuspensionPredicateInner::Id(UuidOperator::Eq(*inserted.record.id)).into(),
            ),
        )
        .await
        .unwrap();

        assert_eq!(suspensions.len(), 1);
        assert_eq!(suspensions[0].record, inserted.record);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn select_fields() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let view = "suspension_to_specimen";

        tokio::join!(
            ensure_fields_are_selectable::<SpecimenField>(&tx, view),
            ensure_fields_are_selectable::<SuspensionField>(&tx, view),
        );
    }
}
