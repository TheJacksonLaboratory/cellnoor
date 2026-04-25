create view cdna_brief as (
    select
        *,
        row('/cdna/' || id)::simple_links as links
    from cdna
);

-- This is going to return a lot of information for a given cDNA, but it's fine because the utility is very high
create view chromium_cdna_full as (
    select
        cdna,
        gp_full as gem_pool,
        array(
            select mes from cdna_measurement as mes
            where mes.cdna_id = cdna.id
        ) as measurements,
        array(
            select prep.prepared_by
            from cdna_preparer as prep
            where prep.cdna_id = cdna.id
        ) as preparers
    from cdna_brief as cdna join gem_pool_full as gp_full on cdna.gem_pool_id = (gp_full.gem_pool).id
);
