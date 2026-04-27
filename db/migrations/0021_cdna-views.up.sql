create view cdna_compact as (
    select
        *,
        row('/cdna/' || id)::simple_links as links
    from cdna
);

create view cdna_to_specimen as (
    select
        cdna,
        gp.parent_specimen,
        gp as gem_pool
    from cdna_compact as cdna join gem_pool_to_specimen as gp on cdna.gem_pool_id = (gp.gem_pool).id
);
