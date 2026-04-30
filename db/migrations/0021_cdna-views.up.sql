create view cdna_to_specimen as (
    select
        cdna,
        gp.specimen,
        gp as gem_pool
    from cdna join gem_pool_to_specimen as gp on cdna.gem_pool_id = (gp.gem_pool).id
);
