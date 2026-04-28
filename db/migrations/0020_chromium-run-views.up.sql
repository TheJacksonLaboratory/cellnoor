create view chromium_run_compact as (
    select
        *,
        row('/chromium-runs/' || id)::simple_links as links
    from chromium_run
);

create view chromium_run_to_assay as (
    select
        chr as chromium_run,
        assay
    from chromium_run_compact as chr join tenx_assay as assay on chr.assay_id = assay.id
);

-- We don't use this type in the view, but we do in the application, so it makes sense to introduce it here
create type suspension_pool_compact_ocm_barcoded as (
    pool suspension_pool_compact,
    ocm_barcode_id text
);

-- This view is complex because from a gem_pool, we can get to a specimen either from a suspension (query 1) or from a
-- suspension_pool (query 2)
create view gem_pool_to_specimen as (
    select
        chip as chip_loading,
        susp.parent_specimen as specimen,
        gp as gem_pool,
        susp as suspension,
        null as suspension_pool
    from chip_loading as chip
    join gem_pool as gp on chip.gem_pool_id = gp.id
    join suspension_to_specimen as susp on chip.suspension_id = (susp.suspension).id

    -- `union all` because we don't need deduplication because we know there are no duplicates
    union all

    select
        chip as chip_loading,
        pool.parent_specimen as specimen,
        gp as gem_pool,
        null as suspension,
        pool
    from chip_loading as chip
    join gem_pool as gp on chip.gem_pool_id = gp.id
    join suspension_pool_to_specimen as pool on chip.suspension_pool_id = (pool.suspension_pool).id
);
