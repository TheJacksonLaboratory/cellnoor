use diesel::prelude::*;
use scamplers_schema::{
    cdna, chip_loadings, chromium_dataset_libraries, chromium_datasets, chromium_runs, gem_pools,
    labs, libraries, specimens, suspension_pools, suspension_tagging, suspensions, tenx_assays,
};

#[diesel::dsl::auto_type]
pub(crate) fn chromium_datasets_to_specimens() -> _ {
    chromium_datasets::table.inner_join(labs::table).inner_join(
        chromium_dataset_libraries::table.inner_join(
            libraries::table.inner_join(
                cdna::table.inner_join(
                    gem_pools::table
                        .inner_join(
                            chip_loadings::table
                                .inner_join(suspensions::table.inner_join(specimens::table)),
                        )
                        .inner_join(chromium_runs::table.inner_join(tenx_assays::table)),
                ),
            ),
        ),
    )
}

#[diesel::dsl::auto_type]
pub(crate) fn chromium_datasets_to_pooled_specimens() -> _ {
    chromium_datasets::table.inner_join(labs::table).inner_join(
        chromium_dataset_libraries::table.inner_join(
            libraries::table.inner_join(
                cdna::table.inner_join(
                    gem_pools::table
                        .inner_join(
                            chip_loadings::table.inner_join(
                                suspension_pools::table.inner_join(
                                    suspension_tagging::table.inner_join(
                                        suspensions::table.inner_join(specimens::table),
                                    ),
                                ),
                            ),
                        )
                        .inner_join(chromium_runs::table.inner_join(tenx_assays::table)),
                ),
            ),
        ),
    )
}
