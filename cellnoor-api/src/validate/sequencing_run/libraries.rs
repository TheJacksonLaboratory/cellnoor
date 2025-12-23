use cellnoor_models::{
    library::{LibraryFilter, LibraryQuery},
    sequencing_run::SequencingRunIdLibraries,
};
use cellnoor_schema::sequencing_runs;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl Validate for (SequencingRunIdLibraries, Vec<Uuid>) {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        let (SequencingRunIdLibraries(sequencing_run_id), library_ids) = self;

        let mut library_query = LibraryQuery::default_with_no_limit();
        library_query.filter = Some(LibraryFilter {
            ids: Some(library_ids.clone()),
        });
        let libraries = library_query.execute(db_conn)?;

        let sequencing_run_begun_at = sequencing_runs::table
            .select(sequencing_runs::begun_at)
            .filter(sequencing_runs::id.eq(sequencing_run_id))
            .first(db_conn)
            .map(jiff_diesel::Timestamp::to_jiff)?;

        for lib in libraries {
            validate_timestamps(sequencing_run_begun_at, lib.prepared_at(), "begun_at")?;
        }

        Ok(())
    }
}
