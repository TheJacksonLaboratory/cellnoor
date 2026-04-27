create view suspension_pool_compact as (
    select
        *,
        row('/suspension-pools/' || id)::simple_links as links
    from suspension_pool
);

-- We don't use this type in the view, but we do in the application, so it makes sense to introduce it here
create type suspension_compact_tagged_or_ocm_barcoded as (
    suspension suspension_compact,
    tag multiplexing_tag,
    ocm_barcode_id text
);

-- We include the multiplexing tag because it's cheap and is useful for consumers
create view suspension_pool_to_specimen as (
    select
        pool as suspension_pool,
        susp.parent_specimen,
        susp as suspension,
        multiplexing_tag
    from suspension_pool_compact as pool
    join suspension_pooling as pooling on pool.id = pooling.pool_id
    join suspension_to_specimen as susp on pooling.suspension_id = (susp.suspension).id
    left join multiplexing_tag on pooling.tag_id = multiplexing_tag.id
);
