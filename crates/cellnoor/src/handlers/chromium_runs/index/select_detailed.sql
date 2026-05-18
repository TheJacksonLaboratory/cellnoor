select distinct on ((chromium_run).id)
    chromium_run,
    tenx_assay,
    array(
        select (
            gw.gem_well,
            array_agg(
                (gw.specimen, gw.multiplexing_tag, gw.ocm_barcode_id)::tagged_specimen
            )
        )::gem_well_with_specimens
        from gem_well_to_specimen as gw
        where (gw.chromium_run).id = (chromium_run).id
        group by gw.gem_well
    ) as gem_wells
from gem_well_to_specimen
where true
group by chromium_run, tenx_assay
