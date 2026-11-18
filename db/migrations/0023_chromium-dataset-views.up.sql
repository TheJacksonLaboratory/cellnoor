create view chromium_dataset_to_specimen with (security_invoker = true) as (
    select
        chromium_dataset,
        reference_package,
        library.library,
        library.specimen,
        library.tenx_assay,
        library.chromium_run,
        library.multiplexing_tag,
        library.ocm_barcode_id,
        library.cdna
    from chromium_dataset
    join reference_package on chromium_dataset.reference_package_name = reference_package.package_name
    join chromium_dataset_library as cds_lib on chromium_dataset.id = cds_lib.dataset_id
    join chromium_library_to_specimen as library on cds_lib.library_id = (library.library).id
);
