use cellnoor_models::tenx_assay::TenxAssayCreation;
use cellnoor_schema::{library_type_specifications, tenx_assays::dsl::*};
use diesel::prelude::*;
use uuid::Uuid;

use crate::initial_data::Upsert;

impl Upsert for TenxAssayCreation {
    fn upsert(self, db_conn: &mut diesel::PgConnection) -> anyhow::Result<()> {
        let lib_type_specs = self.library_type_specifications().map(<[_]>::to_vec);

        let assay_id: Uuid = match self {
            Self::Chromium(a) => {
                let library_type_names = a.library_types();

                diesel::insert_into(tenx_assays)
                    .values((library_types.eq(library_type_names), a.clone()))
                    .on_conflict((name, library_types, sample_multiplexing, chemistry_version))
                    .do_update()
                    .set(a)
                    .returning(id)
                    .get_result(db_conn)?
            }
        };

        let Some(lib_type_specs) = lib_type_specs else {
            return Ok(());
        };

        for spec in &lib_type_specs {
            let values = (library_type_specifications::assay_id.eq(assay_id), spec);

            diesel::insert_into(library_type_specifications::table)
                .values(values)
                .on_conflict((
                    library_type_specifications::assay_id,
                    library_type_specifications::library_type,
                ))
                .do_update()
                .set(values)
                .execute(db_conn)?;
        }

        Ok(())
    }
}
