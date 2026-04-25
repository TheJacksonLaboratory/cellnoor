create type chromium_dataset_links as (
    self text,
    raw_files text []
);

create view chromium_dataset_brief as (
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

-- Same deal, we traverse the entire tree because it's super useful
create view chromium_dataset_full as (
    select
        cds as chromium_dataset,
        array(
            select lib
            from chromium_library_full as lib
            join chromium_dataset_library as cdl on (lib.library).id = cdl.library_id
            where cdl.dataset_id = cds.id
        ) as libraries,
        array(
            select file from chromium_dataset_parsed_file as file
            where file.dataset_id = cds.id
        ) as parsed_files
    from chromium_dataset_brief as cds
);
