create view chromium_dataset_to_specimen as (
    select
        cds as chromium_dataset,
        lib.specimen,
        lib as library
    from chromium_dataset as cds
    join chromium_dataset_library as cds_lib on cds.id = cds_lib.dataset_id
    join library_to_specimen as lib on cds_lib.library_id = (lib.library).id
);
