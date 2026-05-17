select distinct on ((chromium_run).id)
    chromium_run,
    tenx_assay,
    array(
        select (
            gp.gem_pool,
            array_agg(
                (gp.specimen, gp.multiplexing_tag, gp.ocm_barcode_id)::tagged_specimen
            )
        )::gem_pool_with_specimens
        from gem_pool_to_specimen as gp
        where (gp.chromium_run).id = (chromium_run).id
        group by gp.gem_pool
    ) as gem_pools
from gem_pool_to_specimen
where true
group by chromium_run, tenx_assay
