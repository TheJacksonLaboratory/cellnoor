create view chromium_run_brief as (
    select
        *,
        row('/chromium-runs/' || id)::simple_links as links
    from chromium_run
);

create type suspension_full_tagged_or_ocm_barcoded as (
    suspension suspension_full,
    tag multiplexing_tag,
    ocm_barcode_id text
);

create type suspension_pool_full_ocm_barcoded as (
    pool suspension_pool_full,
    ocm_barcode_id text
);

-- We use the views `suspension_full` and `suspension_pool_full` rather than `suspension_brief` and
-- `suspension_pool_brief` because we use this view for super-deep traversal, and it's only returned when we request
-- one specific Chromium run
create view gem_pool_full as (
    select
        gp as gem_pool,
        array(
            -- We know that the suspension is multiplexed on-chip (OCM) if it's in the `chip_loading` table
            select (susp, null, chl.ocm_barcode_id)::suspension_full_tagged_or_ocm_barcoded
            from chip_loading as chl
            join suspension_full as susp on chl.suspension_id = (susp.suspension).id
            where chl.gem_pool_id = gp.id
        ) as suspensions,
        array(
            select (pool, chl.ocm_barcode_id)::suspension_pool_full_ocm_barcoded
            from chip_loading as chl
            join suspension_pool_full as pool on chl.suspension_id = (pool.suspension_pool).id
            where chl.gem_pool_id = gp.id
        ) as suspension_pools
    from gem_pool as gp
);

create view chromium_run_full as (
    select
        chr as chromium_run,
        assay,
        array(
            select gem_pool_full from gem_pool_full
            where (gem_pool_full.gem_pool).chromium_run_id = chr.id
        )
    from chromium_run as chr join tenx_assay as assay on chr.assay_id = assay.id
);
