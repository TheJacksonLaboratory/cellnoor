-- This view is complex because from a gem_well, we can get to a specimen either from a suspension (query 1) or from a
-- suspension_pool (query 2)
create view gem_well_to_specimen with (security_invoker = true) as (
    select
        chromium_run,
        tenx_assay,
        suspension.specimen,
        gem_well,
        chip_loading.ocm_barcode_id,
        null as multiplexing_tag
    from chip_loading
    join gem_well on chip_loading.gem_well_id = gem_well.id
    join chromium_run on gem_well.chromium_run_id = chromium_run.id
    join tenx_assay on chromium_run.assay_id = tenx_assay.id
    join suspension_to_specimen as suspension on chip_loading.suspension_id = (suspension.suspension).id

    -- `union all` because we don't need deduplication because we know there are no duplicates
    union all

    select
        chromium_run,
        tenx_assay,
        suspension_pool.specimen,
        gem_well,
        chip_loading.ocm_barcode_id,
        suspension_pool.multiplexing_tag
    from chip_loading
    join gem_well on chip_loading.gem_well_id = gem_well.id
    join chromium_run on gem_well.chromium_run_id = chromium_run.id
    join tenx_assay on chromium_run.assay_id = tenx_assay.id
    join
        suspension_pool_to_specimen as suspension_pool
        on chip_loading.suspension_pool_id = (suspension_pool.suspension_pool).id
);

create type gem_well_with_specimens as (
    gem_well gem_well,
    specimens tagged_specimen []
);
