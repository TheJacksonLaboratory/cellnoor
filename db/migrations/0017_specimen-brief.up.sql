create view specimen_brief as (
    select
        *,
        row('/specimens/' || id)::simple_links as links
    from specimen
);
