use axum::extract::State;
use diesel::{dsl::AssumeNotNull, prelude::*};
use jiff_diesel::ToDiesel;
use reqwest::StatusCode;
use scamplers_models::specimen::{SpecimenFilter, SpecimenQuery, SpecimenSummary};
use scamplers_schema::specimens::dsl::{
    cryopreserved, embedded_in, fixative, frozen, host_species, id, lab_id, name, received_at,
    returned_at, returned_by, species, submitted_by, tissue, type_,
};
use serde_qs::web::QsQuery;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db::{self, BoxedFilter, BoxedFilterExt, ToBoxedFilter, utils::like_any},
    state::AppState,
};

pub(super) async fn list_specimens(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    QsQuery(request): QsQuery<SpecimenQuery>,
) -> ApiResponse<Vec<SpecimenSummary>> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Vec<SpecimenSummary>> for SpecimenQuery {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<SpecimenSummary>, db::Error> {
        let filter = self.filter();

        let mut stmt = SpecimenSummary::query()
            .limit(self.limit())
            .offset(self.offset())
            .filter(filter.to_boxed_filter())
            .into_boxed();

        for ordering in self.order_by() {
            stmt = stmt.then_order_by(ordering);
        }

        Ok(stmt.load(db_conn)?)
    }
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for SpecimenFilter
where
    id: SelectableExpression<QS>,
    name: SelectableExpression<QS>,
    submitted_by: SelectableExpression<QS>,
    lab_id: SelectableExpression<QS>,
    received_at: SelectableExpression<QS>,
    species: SelectableExpression<QS>,
    AssumeNotNull<host_species>: SelectableExpression<QS>,
    type_: SelectableExpression<QS>,
    AssumeNotNull<tissue>: SelectableExpression<QS>,
    AssumeNotNull<embedded_in>: SelectableExpression<QS>,
    AssumeNotNull<fixative>: SelectableExpression<QS>,
    frozen: SelectableExpression<QS>,
    cryopreserved: SelectableExpression<QS>,
    AssumeNotNull<returned_by>: SelectableExpression<QS>,
    AssumeNotNull<returned_at>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> db::BoxedFilter<'a, QS> {
        let mut filter = BoxedFilter::new();

        if let Some(ids) = self.ids() {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(names) = self.names() {
            filter = filter.and_condition(like_any(name, names));
        }

        if let Some(submitter_list) = self.submitted_by() {
            filter = filter.and_condition(submitted_by.eq_any(submitter_list));
        }

        if let Some(labs) = self.labs() {
            filter = filter.and_condition(lab_id.eq_any(labs));
        }

        if let Some(received_before) = self.received_before().map(ToDiesel::to_diesel) {
            filter = filter.and_condition(received_at.lt(received_before));
        }

        if let Some(received_after) = self.received_after().map(ToDiesel::to_diesel) {
            filter = filter.and_condition(received_at.gt(received_after));
        }

        if let Some(species_list) = self.species() {
            filter = filter.and_condition(species.eq_any(species_list));
        }

        if let Some(host_species_list) = self.host_species() {
            filter = filter.and_condition(host_species.assume_not_null().eq_any(host_species_list));
        }

        if let Some(types) = self.types() {
            filter = filter.and_condition(type_.eq_any(types));
        }

        if let Some(embedding_matrices) = self.embedded_in() {
            filter = filter.and_condition(embedded_in.assume_not_null().eq_any(embedding_matrices));
        }

        if let Some(fixatives) = self.fixatives() {
            filter = filter.and_condition(fixative.assume_not_null().eq_any(fixatives));
        }

        if let Some(is_frozen) = self.frozen() {
            filter = filter.and_condition(frozen.eq(is_frozen));
        }

        if let Some(is_cryopreserved) = self.cryopreserved() {
            filter = filter.and_condition(cryopreserved.eq(is_cryopreserved));
        }

        if let Some(tissues) = self.tissues() {
            filter = filter.and_condition(like_any(tissue.assume_not_null(), tissues));
        }

        if let Some(returner_list) = self.returned_by() {
            filter = filter.and_condition(returned_by.assume_not_null().eq_any(returner_list));
        }

        if let Some(returned_before) = self.returned_before().map(ToDiesel::to_diesel) {
            filter = filter.and_condition(returned_at.assume_not_null().lt(returned_before));
        }

        if let Some(returned_after) = self.returned_after().map(ToDiesel::to_diesel) {
            filter = filter.and_condition(returned_at.assume_not_null().gt(returned_after));
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use deadpool_diesel::postgres::Connection;
    use rstest::rstest;
    use scamplers_models::specimen::*;

    use crate::{
        test_state::{Database, database, root_db_conn},
        test_util::test_query,
    };

    fn sort_by_received_at(i1: &&SpecimenSummary, i2: &&SpecimenSummary) -> Ordering {
        i1.received_at().cmp(&i2.received_at())
    }

    fn sort_by_tissue(i1: &&SpecimenSummary, i2: &&SpecimenSummary) -> Ordering {
        i1.tissue().to_lowercase().cmp(&i2.tissue().to_lowercase())
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_specimen_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        test_query::<SpecimenQuery, _>()
            .all_records(&database.specimens)
            .sort_by(sort_by_received_at)
            .run(root_db_conn)
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_specimen_query(
        #[future] root_db_conn: Connection,
        #[future] database: &'static Database,
    ) {
        let query = SpecimenQuery::builder()
            .filter(
                SpecimenFilter::builder()
                    .names(["%s", "%p%"].map(str::to_owned))
                    .build(),
            )
            .limit(i64::MAX)
            .order_by(SpecimenOrderBy::received_at {
                descending: Some(false),
            })
            .order_by(SpecimenOrderBy::tissue {
                descending: Some(true),
            })
            .build();

        test_query()
            .all_records(&database.specimens)
            .filter(|i| {
                let s = i.name().to_lowercase();
                s.ends_with("s") | s.contains("p")
            })
            .sort_by(|i1, i2| sort_by_received_at(i1, i2).then(sort_by_tissue(i1, i2).reverse()))
            .db_query(query)
            .run(root_db_conn)
            .await;
    }
}
