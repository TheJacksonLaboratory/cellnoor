create type chromium_dataset_links as (
    self text,
    raw_files text []
);

create view chromium_dataset_compact as (
    select
        *,
        row(
            '/chromium-datasets/' || id, array(
                select '/chromium-datasets/' || cds.id || '/raw-files/' || file.path
                from chromium_dataset_raw_file as file
                where file.dataset_id = cds.id
            )
        )::chromium_dataset_links as links
    from chromium_dataset as cds
);

create view chromium_dataset_to_specimen as (
    select
        cds as chromium_dataset,
        lib.specimen,
        lib as library
    from chromium_dataset_compact as cds
    join chromium_dataset_library as cds_lib on cds.id = cds_lib.dataset_id
    join library_to_specimen as lib on cds_lib.library_id = (lib.library).id
);
