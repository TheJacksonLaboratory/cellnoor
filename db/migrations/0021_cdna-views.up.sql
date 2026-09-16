create view chromium_cdna_to_specimen with (security_invoker = true) as (
    select
        cdna,
        gw_ts.specimen,
        gw_ts.tenx_assay,
        gw_ts.multiplexing_tag,
        gw_ts.ocm_barcode_id
    from cdna join gem_well_to_specimen as gw_ts on cdna.gem_well_id = (gw_ts.gem_well).id
);
