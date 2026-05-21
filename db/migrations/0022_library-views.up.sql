create view library_to_specimen as (
    select
        library,
        cdna.specimen,
        -- Bring tenx_assay one level forward because chromium_dataset_to_specimen needs it
        cdna.tenx_assay,
        cdna
    from library join cdna_to_specimen as cdna on library.cdna_id = (cdna.cdna).id
);
