use std::collections::HashSet;

use axum::{extract::State, http::StatusCode};
use cellnoor_models::specimen::{SpecimenFilter, SpecimenQuery, SpecimenSummary};
use cellnoor_schema::specimens as t;
use diesel::{dsl::AssumeNotNull, prelude::*};
use jiff_diesel::ToDiesel;
use uuid::Uuid;

use crate::{
    api::{
        auth::{self, Authorization},
        extract::{QsQuery, auth::AuthenticatedUser},
        routes::handle_api_request,
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter, like_any},
    state::AppState,
};

// pub(super) async fn list_specimens(
//     _: Root,
//     state: State<AppState>,
//     user: AuthenticatedUser,
//     QsQuery(request): QsQuery<SpecimenQuery>,
// ) -> ApiResponse<Vec<SpecimenSummary>> {
//     let item = handle_api_request(state, user, request).await?;
//     Ok((StatusCode::OK, item))
// }

// impl db::Operation<Vec<SpecimenSummary>> for SpecimenQuery {
//     type ValidationData = ();

//     async fn fetch_validation_data(
//         &self,
//         _state: &DbConnection,
//     ) -> Result<Self::ValidationData, db::Error> {
//         Ok(())
//     }

//     fn execute(
//         self,
//         user: AuthorizationData,
//         _validation_data: (),
//         db_conn: &mut diesel::PgConnection,
//     ) -> Result<Vec<SpecimenSummary>, db::Error> {
//         let requested_projects = self.filter.projects.take();
//         self.filter.projects = user.authorized_projects(requested_projects);

//         let Self {
//             filter,
//             limit,
//             offset,
//             order_by,
//         } = self;

//         let mut stmt = SpecimenSummary::query()
//             .limit(limit)
//             .offset(offset)
//             .filter(filter.to_boxed_filter())
//             .into_boxed();

//         for ordering in order_by.as_ref() {
//             stmt = stmt.then_order_by(ordering);
//         }

//         Ok(stmt.load(db_conn)?)
//     }
// }

// In order to be composed into a `ChromiumDatasetFilter`, we need calls to
// `assume_not_null`, which has no essentially no runtime impact
impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for SpecimenFilter
where
    AssumeNotNull<t::id>: SelectableExpression<QS>,
    AssumeNotNull<t::name>: SelectableExpression<QS>,
    AssumeNotNull<t::submitted_by>: SelectableExpression<QS>,
    AssumeNotNull<t::project_id>: SelectableExpression<QS>,
    AssumeNotNull<t::received_at>: SelectableExpression<QS>,
    AssumeNotNull<t::species>: SelectableExpression<QS>,
    AssumeNotNull<t::host_species>: SelectableExpression<QS>,
    AssumeNotNull<t::type_>: SelectableExpression<QS>,
    AssumeNotNull<t::tissue>: SelectableExpression<QS>,
    AssumeNotNull<t::embedded_in>: SelectableExpression<QS>,
    AssumeNotNull<t::fixative>: SelectableExpression<QS>,
    AssumeNotNull<t::thermal_preservation_method>: SelectableExpression<QS>,
    AssumeNotNull<t::returned_by>: SelectableExpression<QS>,
    AssumeNotNull<t::returned_at>: SelectableExpression<QS>,
    AssumeNotNull<t::additional_data>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> db::BoxedFilter<'a, QS> {
        let mut filter = BoxedFilter::new_true();

        let Self {
            ids,
            names,
            submitted_by,
            projects,
            received_before,
            received_after,
            species,
            host_species,
            types,
            embedded_in,
            fixatives,
            fresh,
            thermal_preservation_methods,
            tissues,
            returned_before,
            returned_after,
            returned_by,
            additional_data,
        } = self;

        if let Some(ids) = ids {
            filter = filter.and_condition(t::id.assume_not_null().eq_any(ids));
        }

        if let Some(names) = names {
            filter = filter.and_condition(like_any(t::name.assume_not_null(), names));
        }

        if let Some(submitter_list) = submitted_by {
            filter = filter.and_condition(t::submitted_by.assume_not_null().eq_any(submitter_list));
        }

        if let Some(projects) = projects {
            filter = filter.and_condition(t::project_id.assume_not_null().eq_any(projects));
        }

        if let Some(received_before) = received_before.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(t::received_at.assume_not_null().lt(received_before));
        }

        if let Some(received_after) = received_after.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(t::received_at.assume_not_null().gt(received_after));
        }

        if let Some(species_list) = species {
            filter = filter.and_condition(t::species.assume_not_null().eq_any(species_list));
        }

        if let Some(host_species_list) = host_species {
            filter =
                filter.and_condition(t::host_species.assume_not_null().eq_any(host_species_list));
        }

        if let Some(types) = types {
            filter = filter.and_condition(t::type_.assume_not_null().eq_any(types));
        }

        if let Some(embedding_matrices) = embedded_in {
            filter =
                filter.and_condition(t::embedded_in.assume_not_null().eq_any(embedding_matrices));
        }

        if let Some(fixatives) = fixatives {
            filter = filter.and_condition(t::fixative.assume_not_null().eq_any(fixatives));
        }

        if let Some(thermal_preservation_methods) = thermal_preservation_methods {
            filter = filter.and_condition(
                t::thermal_preservation_method
                    .assume_not_null()
                    .eq_any(thermal_preservation_methods),
            );
        }

        // There are some calls to `assume_not_null` to satisfy the type-system. This
        // has no impact on the generated SQL.
        if let Some(true) = fresh {
            filter = filter
                .and_condition(t::thermal_preservation_method.assume_not_null().is_null())
                .and_condition(t::fixative.assume_not_null().is_null());
        } else if let Some(false) = fresh {
            filter = filter.and_condition(
                t::thermal_preservation_method
                    .assume_not_null()
                    .is_not_null()
                    .or(t::fixative.assume_not_null().is_not_null()),
            );
        }

        if let Some(tissues) = tissues {
            filter = filter.and_condition(like_any(t::tissue.assume_not_null(), tissues));
        }

        if let Some(returner_list) = returned_by {
            filter = filter.and_condition(t::returned_by.assume_not_null().eq_any(returner_list));
        }

        if let Some(returned_before) = returned_before.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(t::returned_at.assume_not_null().lt(returned_before));
        }

        if let Some(returned_after) = returned_after.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(t::returned_at.assume_not_null().gt(returned_after));
        }

        if let Some(additional_data) = additional_data {
            filter = filter.and_condition(
                t::additional_data
                    .assume_not_null()
                    .contains(additional_data),
            );
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use cellnoor_models::specimen::*;
    use rstest::rstest;

    use crate::{
        db::DbConnection,
        test_state::{Database, database, root_db_conn},
        test_util::test_query,
    };

    fn sort_by_received_at(i1: &&SpecimenSummary, i2: &&SpecimenSummary) -> Ordering {
        i2.received_at().cmp(&i1.received_at())
    }

    fn sort_by_tissue(i1: &&SpecimenSummary, i2: &&SpecimenSummary) -> Ordering {
        i1.tissue().to_lowercase().cmp(&i2.tissue().to_lowercase())
    }

    #[rstest]
    #[awt]
    #[tokio::test(flavor = "multi_thread")]
    async fn default_specimen_query(
        #[future] root_db_conn: DbConnection,
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
    #[tokio::test(flavor = "multi_thread")]
    async fn specific_specimen_query(
        #[future] root_db_conn: DbConnection,
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
            .sort_by(|i1, i2| {
                sort_by_received_at(i1, i2)
                    .reverse()
                    .then(sort_by_tissue(i1, i2).reverse())
            })
            .db_query(query)
            .run(root_db_conn)
            .await;
    }
}
