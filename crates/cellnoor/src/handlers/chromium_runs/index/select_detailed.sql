with gem_wells as (
    select
        chromium_run,
        tenx_assay,
        (
            gem_well,
            array_agg((specimen, multiplexing_tag, ocm_barcode_id)::tagged_specimen)
        )::gem_well_with_specimens as gem_well
    from gem_well_to_specimen
    /* {where} */
    group by chromium_run, tenx_assay, gem_well
)

select
    chromium_run,
    tenx_assay,
    array_agg(gem_well) as gem_wells
from gem_wells
group by chromium_run, tenx_assay
