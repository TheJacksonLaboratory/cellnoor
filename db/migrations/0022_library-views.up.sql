create view library_to_specimen as (
    select
        library,
        -- Bring the following columns forward because they're useful
        cdna_ts as cdna,
        cdna_ts.specimen,
        cdna_ts.tenx_assay,
        cdna_ts.multiplexing_tag,
        cdna_ts.ocm_barcode_id
    from library join cdna_to_specimen as cdna_ts on library.cdna_id = (cdna_ts.cdna).id
);
