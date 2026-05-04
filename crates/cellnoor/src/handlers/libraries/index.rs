use axum::{Json, extract::State};
use cellnoor_models::library::{LibraryFilter, LibraryQuery, LibrarySummary};
use cellnoor_schema::{
    cdna,
    libraries::{
        additional_data, cdna_id, dual_index_set_name, id, number_of_sample_index_pcr_cycles,
        prepared_at, project_id, readable_id, single_index_set_name, target_reads_per_cell,
    },
};
use diesel::{SelectableExpression, dsl::AssumeNotNull, prelude::*};
use diesel_async::RunQueryDsl;
use jiff_diesel::ToDiesel;

use crate::{
    api::{
        auth::RemoveUnauthorizedProjects,
        extract::{AuthJsonQuery, Authorize},
    },
    db::{self, BoxedFilter, BoxedFilterExt, DbConnection, ToBoxedFilter, like_any},
    state::AppState,
};

pub(super) async fn index_libraries(
    _: State<AppState>,
    db_conn: DbConnection,
    AuthJsonQuery { q }: AuthJsonQuery<LibraryQuery>,
) -> Result<Json<Vec<LibrarySummary>>, db::Error> {
    select_libraries(q, &db_conn).await.map(Json)
}

pub async fn select_libraries(
    LibraryQuery {
        filter,
        limit,
        offset,
        order_by,
    }: LibraryQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<LibrarySummary>, db::Error> {
    let mut stmt = LibrarySummary::query()
        .limit(limit)
        .offset(offset)
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}

impl<'a, QS: 'a> ToBoxedFilter<'a, QS> for LibraryFilter
where
    id: SelectableExpression<QS>,
    readable_id: SelectableExpression<QS>,
    cdna_id: SelectableExpression<QS>,
    project_id: SelectableExpression<QS>,
    AssumeNotNull<single_index_set_name>: SelectableExpression<QS>,
    AssumeNotNull<dual_index_set_name>: SelectableExpression<QS>,
    number_of_sample_index_pcr_cycles: SelectableExpression<QS>,
    AssumeNotNull<target_reads_per_cell>: SelectableExpression<QS>,
    prepared_at: SelectableExpression<QS>,
    cdna::library_type: SelectableExpression<QS>,
    AssumeNotNull<additional_data>: SelectableExpression<QS>,
{
    fn to_boxed_filter(&'a self) -> BoxedFilter<'a, QS> {
        let Self {
            ids,
            readable_ids,
            cdna_ids,
            project_ids,
            single_index_set_names,
            dual_index_set_names,
            number_of_sample_index_pcr_cycles_less_than,
            number_of_sample_index_pcr_cycles_more_than,
            target_reads_per_cell_less_than,
            target_reads_per_cell_more_than,
            prepared_before,
            prepared_after,
            library_types,
            additional_data: additional_data_filter,
        } = self;
        let mut filter = BoxedFilter::new_true();

        if let Some(ids) = ids {
            filter = filter.and_condition(id.eq_any(ids));
        }

        if let Some(readable_ids) = readable_ids {
            filter = filter.and_condition(like_any(readable_id, readable_ids));
        }

        if let Some(cdna_ids) = cdna_ids {
            filter = filter.and_condition(cdna_id.eq_any(cdna_ids));
        }

        if let Some(project_ids) = project_ids {
            filter = filter.and_condition(project_id.eq_any(project_ids));
        }

        if let Some(single_index_set_names) = single_index_set_names {
            filter = filter.and_condition(
                single_index_set_name
                    .assume_not_null()
                    .eq_any(single_index_set_names),
            );
        }

        if let Some(dual_index_set_names) = dual_index_set_names {
            filter = filter.and_condition(
                dual_index_set_name
                    .assume_not_null()
                    .eq_any(dual_index_set_names),
            );
        }

        if let Some(number_of_sample_index_pcr_cycles_less_than) =
            number_of_sample_index_pcr_cycles_less_than
        {
            filter = filter.and_condition(
                number_of_sample_index_pcr_cycles.lt(*number_of_sample_index_pcr_cycles_less_than),
            );
        }

        if let Some(number_of_sample_index_pcr_cycles_more_than) =
            number_of_sample_index_pcr_cycles_more_than
        {
            filter = filter.and_condition(
                number_of_sample_index_pcr_cycles.gt(*number_of_sample_index_pcr_cycles_more_than),
            );
        }

        if let Some(target_reads_per_cell_less_than) = target_reads_per_cell_less_than {
            filter = filter.and_condition(
                target_reads_per_cell
                    .assume_not_null()
                    .lt(*target_reads_per_cell_less_than),
            );
        }

        if let Some(target_reads_per_cell_more_than) = target_reads_per_cell_more_than {
            filter = filter.and_condition(
                target_reads_per_cell
                    .assume_not_null()
                    .gt(*target_reads_per_cell_more_than),
            );
        }

        if let Some(prepared_before) = prepared_before.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(prepared_at.lt(prepared_before));
        }

        if let Some(prepared_after) = prepared_after.map(ToDiesel::to_diesel) {
            filter = filter.and_condition(prepared_at.gt(prepared_after));
        }

        if let Some(library_types) = library_types {
            filter = filter.and_condition(cdna::library_type.eq_any(library_types));
        }

        if let Some(additional_data_filter) = additional_data_filter {
            filter = filter.and_condition(
                additional_data
                    .assume_not_null()
                    .contains(additional_data_filter),
            );
        }

        filter
    }
}

impl Authorize for LibraryQuery {
    fn authorize(
        mut self,
        user: &crate::api::auth::AuthUser,
    ) -> Result<Self, crate::api::auth::Error> {
        self.filter.project_ids.remove_unauthorized_projects(user);

        Ok(self)
    }
}
