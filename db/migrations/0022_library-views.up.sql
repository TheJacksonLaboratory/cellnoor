create view library_brief as (
    select
        *,
        row('/libraries/' || id)::simple_links as links
    from library
);

-- Again, we are okay traversing the entire tree from library up to dataset because the utility of seeing the specimens that went into a library is very high
create view chromium_library_full as (
    select
        lib as library,
        cdna,
        array(
            select mes from library_measurement as mes
            where mes.library_id = lib.id
        ) as measurements,
        array(
            select prep.prepared_by
            from library_preparer as prep
            where prep.library_id = lib.id
        ) as preparers
    from library_brief as lib join chromium_cdna_full as cdna on lib.cdna_id = (cdna.cdna).id
);
