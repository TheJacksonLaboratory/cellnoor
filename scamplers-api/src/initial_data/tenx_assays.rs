use diesel::prelude::*;
use scamplers_models::tenx_assay::{TenxAssay, TenxAssayCreation};
use scamplers_schema::tenx_assays::dsl::*;
use uuid::Uuid;

use crate::initial_data::Upsert;

impl Upsert for TenxAssayCreation {
    fn upsert(self, db_conn: &mut diesel::PgConnection) -> anyhow::Result<()> {
        let assay_id: Uuid = match self {
            Self::Chromium(a) => {
                let library_type_names = a.library_types();

                diesel::insert_into(tenx_assays)
                    .values((library_types.eq(library_type_names), a))
                    .returning(id)
                    .get_result(db_conn)?
            }
        };

        Ok(())
    }
}
